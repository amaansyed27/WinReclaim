const REPOSITORY = "amaansyed27/WinReclaim";
const RELEASE_PAGE = `https://github.com/${REPOSITORY}/releases/latest`;

const header = document.querySelector("[data-header]");
const menuButton = document.querySelector("[data-menu-button]");
const navigation = document.querySelector("[data-nav]");

function updateHeader() {
  header?.classList.toggle("is-scrolled", window.scrollY > 12);
}

function closeMenu() {
  if (!menuButton || !navigation) return;
  menuButton.setAttribute("aria-expanded", "false");
  menuButton.setAttribute("aria-label", "Open navigation");
  navigation.classList.remove("is-open");
  document.body.classList.remove("menu-open");
}

menuButton?.addEventListener("click", () => {
  const opening = menuButton.getAttribute("aria-expanded") !== "true";
  menuButton.setAttribute("aria-expanded", String(opening));
  menuButton.setAttribute("aria-label", opening ? "Close navigation" : "Open navigation");
  navigation?.classList.toggle("is-open", opening);
  document.body.classList.toggle("menu-open", opening);
});

navigation?.querySelectorAll("a").forEach((link) => {
  link.addEventListener("click", closeMenu);
});

window.addEventListener("scroll", updateHeader, { passive: true });
window.addEventListener("resize", () => {
  if (window.innerWidth > 860) closeMenu();
});
updateHeader();

const revealElements = document.querySelectorAll(".reveal");
const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

if (reducedMotion || !("IntersectionObserver" in window)) {
  revealElements.forEach((element) => element.classList.add("is-visible"));
} else {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        entry.target.classList.add("is-visible");
        observer.unobserve(entry.target);
      });
    },
    { threshold: 0.12, rootMargin: "0px 0px -45px" }
  );

  revealElements.forEach((element) => observer.observe(element));
}

function formatBytes(bytes) {
  if (!Number.isFinite(bytes) || bytes <= 0) return "";
  const units = ["B", "KB", "MB", "GB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** index;
  return `${value >= 10 || index === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[index]}`;
}

function formatReleaseDate(value) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  return new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric"
  }).format(date);
}

function selectAsset(assets, extension, preferredPattern) {
  const candidates = assets.filter((asset) => {
    const name = asset.name.toLowerCase();
    return name.endsWith(extension) && !name.endsWith(`${extension}.sig`);
  });

  return candidates.find((asset) => preferredPattern.test(asset.name)) ?? candidates[0] ?? null;
}

function updateLinks(selector, asset, fallback = RELEASE_PAGE) {
  document.querySelectorAll(selector).forEach((link) => {
    link.href = asset?.browser_download_url ?? fallback;
    if (asset) link.setAttribute("download", "");
    else link.removeAttribute("download");
  });
}

async function loadLatestRelease() {
  try {
    const response = await fetch(`https://api.github.com/repos/${REPOSITORY}/releases/latest`, {
      headers: { Accept: "application/vnd.github+json" }
    });

    if (!response.ok) throw new Error(`GitHub returned ${response.status}`);

    const release = await response.json();
    const assets = Array.isArray(release.assets) ? release.assets : [];
    const exe = selectAsset(assets, ".exe", /(x64|setup)/i);
    const msi = selectAsset(assets, ".msi", /x64/i);
    const version = String(release.tag_name || "").replace(/^v/i, "");

    updateLinks("[data-download-exe]", exe);
    updateLinks("[data-download-msi]", msi);

    document.querySelectorAll("[data-download-label]").forEach((label) => {
      label.textContent = version ? `Download v${version} for Windows` : "Download for Windows";
    });

    document.querySelectorAll("[data-version]").forEach((element) => {
      element.textContent = version ? `v${version}` : "Latest release";
    });

    const details = [
      "Windows 11 x64",
      version ? `v${version}` : "latest stable release",
      exe?.size ? formatBytes(exe.size) : "",
      release.published_at ? formatReleaseDate(release.published_at) : ""
    ].filter(Boolean);

    document.querySelectorAll("[data-release-meta]").forEach((element) => {
      element.textContent = details.join(" · ");
    });
  } catch (error) {
    console.info("Latest WinReclaim release could not be resolved; using the releases page fallback.", error);
    updateLinks("[data-download-exe]", null);
    updateLinks("[data-download-msi]", null);
  }
}

document.querySelectorAll("[data-year]").forEach((element) => {
  element.textContent = String(new Date().getFullYear());
});

loadLatestRelease();
