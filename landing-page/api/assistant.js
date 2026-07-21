const OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions";
const MODEL = "openrouter/free";
const APP_URL = "https://winreclaim.vercel.app";
const MAX_BODY_CHARS = 80_000;
const RATE_WINDOW_MS = 60 * 60 * 1000;
const RATE_LIMIT = 20;
const MAX_RATE_KEYS = 5_000;
const requestsByIp = new Map();

const storageSchema = {
  type: "object",
  additionalProperties: false,
  properties: {
    summary: {
      type: "string",
      description: "Two to four concise sentences describing the storage picture without recommending deletion."
    },
    observations: {
      type: "array",
      maxItems: 6,
      items: { type: "string" }
    }
  },
  required: ["summary", "observations"]
};

export default async function handler(req, res) {
  res.setHeader("Cache-Control", "no-store");
  res.setHeader("X-Content-Type-Options", "nosniff");

  if (req.method !== "POST") {
    res.setHeader("Allow", "POST");
    return res.status(405).json({ error: "Method not allowed" });
  }

  const client = String(req.headers["x-winreclaim-client"] || "");
  if (!/^desktop\/\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(client)) {
    return res.status(403).json({ error: "Unsupported client" });
  }

  const ip = clientIp(req);
  if (!allowRequest(ip)) {
    return res.status(429).json({ error: "Demo rate limit reached. Try again later." });
  }

  const apiKey = process.env.OPENROUTER_API_KEY;
  if (!apiKey) {
    return res.status(503).json({ error: "Cloud assistant is not configured" });
  }

  let body;
  try {
    body = typeof req.body === "string" ? JSON.parse(req.body) : req.body;
  } catch {
    return res.status(400).json({ error: "Invalid JSON body" });
  }

  if (!body || typeof body !== "object" || Array.isArray(body)) {
    return res.status(400).json({ error: "Invalid request body" });
  }

  let serializedBody;
  try {
    serializedBody = JSON.stringify(body);
  } catch {
    return res.status(400).json({ error: "Invalid request body" });
  }
  if (serializedBody.length > MAX_BODY_CHARS) {
    return res.status(413).json({ error: "Request is too large" });
  }

  const bodyKeys = Object.keys(body).sort();
  if (bodyKeys.length !== 2 || bodyKeys[0] !== "data" || bodyKeys[1] !== "task") {
    return res.status(400).json({ error: "Unexpected request fields" });
  }

  let request;
  try {
    request = buildRequest(body.task, body.data);
  } catch (error) {
    return res.status(400).json({ error: error instanceof Error ? error.message : "Invalid request" });
  }

  let upstream;
  try {
    upstream = await fetch(OPENROUTER_URL, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${apiKey}`,
        "Content-Type": "application/json",
        "HTTP-Referer": APP_URL,
        "X-OpenRouter-Title": "WinReclaim"
      },
      body: JSON.stringify({
        model: MODEL,
        messages: request.messages,
        response_format: {
          type: "json_schema",
          json_schema: {
            name: request.schemaName,
            strict: true,
            schema: request.schema
          }
        },
        provider: { require_parameters: true },
        temperature: 0.1,
        max_tokens: request.maxTokens,
        stream: false
      })
    });
  } catch (error) {
    console.error("OpenRouter request failed", boundedError(error));
    return res.status(502).json({ error: "The model router could not be reached" });
  }

  const raw = await upstream.text();
  if (!upstream.ok) {
    console.error("OpenRouter rejected request", upstream.status, boundedText(raw, 600));
    const status = upstream.status === 429 ? 429 : 502;
    return res.status(status).json({
      error: upstream.status === 429
        ? "Free model capacity is busy. Try again shortly."
        : "The free model router could not complete this request."
    });
  }

  try {
    const envelope = JSON.parse(raw);
    const content = completionText(envelope?.choices?.[0]?.message?.content);
    const result = JSON.parse(content);
    validateResult(body.task, result, request.allowedCandidateIds);
    return res.status(200).json({
      model: String(envelope.model || MODEL).slice(0, 160),
      result
    });
  } catch (error) {
    console.error("Invalid structured model response", boundedError(error));
    return res.status(502).json({ error: "The routed model returned an invalid structured response" });
  }
}

function buildRequest(task, data) {
  if (task === "storage_summary") {
    const safeData = normalizeStoragePayload(data);
    return {
      schemaName: "winreclaim_storage_summary",
      schema: storageSchema,
      maxTokens: 650,
      allowedCandidateIds: null,
      messages: [
        {
          role: "system",
          content: [
            "You are the advisory WinReclaim Storage Assistant.",
            "Analyze only the aggregate storage metadata supplied by WinReclaim.",
            "Never claim anything is safe to delete or remove.",
            "Never recommend deletion, uninstallation, commands, cleanup actions, or automatic selection.",
            "Never reinterpret measured sizes, risk counts, action counts, or protection state.",
            "Category rows may overlap, so drive used and free totals are authoritative.",
            "Use cautious language and describe uncertainty."
          ].join(" ")
        },
        {
          role: "user",
          content: `Summarize this anonymized Windows storage scan metadata:\n${JSON.stringify(safeData)}`
        }
      ]
    };
  }

  if (task === "intent_constraints") {
    const safeData = normalizeIntentPayload(data);
    const candidateIds = safeData.candidates.map((candidate) => candidate.candidate_id);
    return {
      schemaName: "winreclaim_intent_constraints",
      schema: intentSchema(candidateIds),
      maxTokens: 500,
      allowedCandidateIds: new Set(candidateIds),
      messages: [
        {
          role: "system",
          content: [
            "Interpret a Windows storage reclaim request into conservative constraints only.",
            "Never invent paths, commands, candidates, or risk classes.",
            "Never permit protected data.",
            "Default to safe_now when risk tolerance is ambiguous.",
            "Permit rebuild_or_redownload only when the user accepts later downloads or rebuilds.",
            "Permit review_first only when the user explicitly asks for a named review category.",
            "Exclusions must be conservative."
          ].join(" ")
        },
        {
          role: "user",
          content: `User request:\n${safeData.prompt}\n\nAnonymized cleanup candidates:\n${JSON.stringify(safeData.candidates)}`
        }
      ]
    };
  }

  throw new Error("Unknown assistant task");
}

function intentSchema(candidateIds) {
  return {
    type: "object",
    additionalProperties: false,
    properties: {
      target_reclaim_bytes: {
        anyOf: [{ type: "integer", minimum: 0 }, { type: "null" }]
      },
      allowed_risk_classes: {
        type: "array",
        uniqueItems: true,
        items: {
          type: "string",
          enum: ["safe_now", "rebuild_or_redownload", "review_first"]
        }
      },
      excluded_candidate_ids: {
        type: "array",
        uniqueItems: true,
        items: { type: "string", enum: candidateIds }
      },
      summary: { type: "string" }
    },
    required: [
      "target_reclaim_bytes",
      "allowed_risk_classes",
      "excluded_candidate_ids",
      "summary"
    ]
  };
}

function normalizeStoragePayload(data) {
  if (!data || typeof data !== "object" || Array.isArray(data)) {
    throw new Error("Storage metadata is required");
  }

  const numberFields = ["used_bytes", "free_bytes", "total_bytes", "drive_count", "scanned_entries", "skipped_entries"];
  for (const field of numberFields) {
    if (!Number.isFinite(data[field]) || data[field] < 0) throw new Error(`Invalid ${field}`);
  }
  if (typeof data.rows_may_overlap !== "boolean") throw new Error("Invalid rows_may_overlap");
  if (!Array.isArray(data.categories) || data.categories.length > 40) {
    throw new Error("Invalid storage categories");
  }

  const categories = data.categories.map((category) => {
    if (!category || typeof category !== "object" || Array.isArray(category)) {
      throw new Error("Invalid storage category");
    }
    if (typeof category.category !== "string" || !category.category.trim() || category.category.length > 80) {
      throw new Error("Invalid storage category");
    }
    for (const field of ["bytes", "locations", "actionable_locations"]) {
      if (!Number.isFinite(category[field]) || category[field] < 0) throw new Error(`Invalid category ${field}`);
    }
    const risks = category.risk_counts;
    if (!risks || typeof risks !== "object" || Array.isArray(risks)) throw new Error("Invalid risk counts");
    const riskCounts = {};
    for (const field of ["safe_now", "rebuild_or_redownload", "review_first", "protected"]) {
      if (!Number.isFinite(risks[field]) || risks[field] < 0) throw new Error(`Invalid risk count ${field}`);
      riskCounts[field] = risks[field];
    }
    return {
      category: category.category.trim(),
      bytes: category.bytes,
      locations: category.locations,
      actionable_locations: category.actionable_locations,
      risk_counts: riskCounts
    };
  });

  return {
    used_bytes: data.used_bytes,
    free_bytes: data.free_bytes,
    total_bytes: data.total_bytes,
    drive_count: data.drive_count,
    scanned_entries: data.scanned_entries,
    skipped_entries: data.skipped_entries,
    rows_may_overlap: data.rows_may_overlap,
    categories
  };
}

function normalizeIntentPayload(data) {
  if (!data || typeof data !== "object" || Array.isArray(data)) {
    throw new Error("Intent metadata is required");
  }
  if (typeof data.prompt !== "string" || !data.prompt.trim() || data.prompt.length > 1000) {
    throw new Error("Invalid reclaim request");
  }
  if (!Array.isArray(data.candidates) || !data.candidates.length || data.candidates.length > 200) {
    throw new Error("Invalid cleanup candidates");
  }

  const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  const seenIds = new Set();
  const candidates = data.candidates.map((candidate) => {
    if (!candidate || typeof candidate !== "object" || Array.isArray(candidate)) {
      throw new Error("Invalid cleanup candidate");
    }
    const id = String(candidate.candidate_id || "");
    if (!uuid.test(id) || seenIds.has(id)) throw new Error("Invalid candidate ID");
    seenIds.add(id);
    if (typeof candidate.category !== "string" || !candidate.category.trim() || candidate.category.length > 100) {
      throw new Error("Invalid candidate category");
    }
    if (!Number.isFinite(candidate.size_bytes) || candidate.size_bytes < 0) {
      throw new Error("Invalid candidate size");
    }
    if (!["safe_now", "rebuild_or_redownload", "review_first"].includes(candidate.risk_class)) {
      throw new Error("Invalid candidate risk class");
    }
    if (!["temporary_or_disposable", "requires_rebuild_or_redownload", "may_disrupt_environment"].includes(candidate.consequence)) {
      throw new Error("Invalid candidate consequence");
    }
    return {
      candidate_id: id,
      category: candidate.category.trim(),
      size_bytes: candidate.size_bytes,
      risk_class: candidate.risk_class,
      consequence: candidate.consequence
    };
  });

  return { prompt: data.prompt.trim(), candidates };
}

function validateResult(task, result, allowedCandidateIds) {
  if (!result || typeof result !== "object" || Array.isArray(result)) {
    throw new Error("Structured result is missing");
  }

  if (task === "storage_summary") {
    assertExactKeys(result, ["observations", "summary"]);
    if (typeof result.summary !== "string" || result.summary.length < 20 || result.summary.length > 1000) {
      throw new Error("Invalid summary");
    }
    if (!Array.isArray(result.observations) || result.observations.length > 6 || result.observations.some((item) => typeof item !== "string" || item.length > 400)) {
      throw new Error("Invalid observations");
    }
    return;
  }

  assertExactKeys(result, ["allowed_risk_classes", "excluded_candidate_ids", "summary", "target_reclaim_bytes"]);
  if (!Array.isArray(result.allowed_risk_classes) || result.allowed_risk_classes.some((value) => !["safe_now", "rebuild_or_redownload", "review_first"].includes(value))) {
    throw new Error("Invalid allowed risk classes");
  }
  if (!Array.isArray(result.excluded_candidate_ids) || result.excluded_candidate_ids.some((value) => !allowedCandidateIds?.has(value))) {
    throw new Error("Invalid excluded candidate IDs");
  }
  if (result.target_reclaim_bytes !== null && (!Number.isInteger(result.target_reclaim_bytes) || result.target_reclaim_bytes < 0)) {
    throw new Error("Invalid reclaim target");
  }
  if (typeof result.summary !== "string" || result.summary.length < 8 || result.summary.length > 500) {
    throw new Error("Invalid intent summary");
  }
}

function assertExactKeys(value, expected) {
  const actual = Object.keys(value).sort();
  if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) {
    throw new Error("Structured response contained unexpected fields");
  }
}

function completionText(content) {
  if (typeof content === "string") return content;
  if (Array.isArray(content)) {
    return content
      .map((part) => typeof part === "string" ? part : part?.text || "")
      .join("");
  }
  throw new Error("Completion content was empty");
}

function clientIp(req) {
  const forwarded = String(req.headers["x-forwarded-for"] || req.headers["x-vercel-forwarded-for"] || "unknown");
  return forwarded.split(",")[0].trim().slice(0, 128);
}

function allowRequest(ip) {
  const now = Date.now();
  if (requestsByIp.size >= MAX_RATE_KEYS) {
    for (const [key, value] of requestsByIp) {
      if (now - value.startedAt >= RATE_WINDOW_MS) requestsByIp.delete(key);
    }
    if (requestsByIp.size >= MAX_RATE_KEYS) requestsByIp.clear();
  }

  const existing = requestsByIp.get(ip);
  if (!existing || now - existing.startedAt >= RATE_WINDOW_MS) {
    requestsByIp.set(ip, { startedAt: now, count: 1 });
    return true;
  }
  if (existing.count >= RATE_LIMIT) return false;
  existing.count += 1;
  return true;
}

function boundedError(error) {
  return String(error instanceof Error ? error.message : error).slice(0, 500);
}

function boundedText(value, max) {
  return String(value).replace(/Bearer\s+[A-Za-z0-9._-]+/gi, "Bearer [redacted]").slice(0, max);
}
