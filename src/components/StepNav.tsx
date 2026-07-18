import type { AppStep } from "../App";

const steps: { id: AppStep; label: string }[] = [
  { id: "scan", label: "Scan" },
  { id: "findings", label: "Findings" },
  { id: "plan", label: "Review" },
  { id: "receipt", label: "Receipt" }
];

export function StepNav({ current }: { current: AppStep }) {
  const activeIndex = steps.findIndex((step) => step.id === current);

  return (
    <nav className="step-nav" aria-label="Progress">
      {steps.map((step, index) => (
        <div
          className={`step-nav-item ${index <= activeIndex ? "is-active" : ""}`}
          key={step.id}
        >
          <span>{String(index + 1).padStart(2, "0")}</span>
          <strong>{step.label}</strong>
        </div>
      ))}
    </nav>
  );
}
