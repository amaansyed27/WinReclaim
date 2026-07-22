# Accessibility Guidance

WinReclaim presents safety-sensitive information. Consequences, warnings and action state must remain understandable without relying on colour, hover or animation alone.

## Goals

The desktop interface should support:

- keyboard-only navigation;
- visible focus indication;
- meaningful screen-reader labels;
- sufficient contrast;
- scalable text and window resizing;
- reduced-motion preferences;
- plain-language consequences;
- non-colour indicators for safety classes and status.

## Keyboard interaction

Interactive controls must:

- be reachable in a logical tab order;
- use native buttons, links, inputs and disclosure elements where possible;
- activate with expected keyboard keys;
- avoid focus traps;
- return focus sensibly after dialogs or page changes;
- keep destructive confirmation controls distinguishable from navigation.

Custom segmented controls and finding rows need explicit roles, labels and selected state.

## Focus visibility

Do not remove the browser/webview focus outline without providing an equal or stronger replacement. Focus indicators should remain visible on dark and light surfaces and must not depend on subtle colour changes alone.

## Screen-reader content

Provide accessible names for:

- scan buttons and cancellation;
- drive selection controls;
- progress bars;
- safety and recovery classes;
- finding selection;
- plan totals and consequences;
- vault expiry and restore state;
- updater status;
- assistant installation progress.

Decorative icons should be hidden from assistive technology. Meaningful icons require text or an accessible label.

## Safety communication

Every cleanup action should expose in text:

- what data is affected;
- whether the action is reversible, rebuildable, redownloadable or irreversible;
- whether files can be skipped;
- whether the action is selected;
- whether protected items are excluded.

Colour can reinforce these classes but cannot be the only indicator.

## Progress and long operations

Scan, download and cleanup progress should:

- include a text phase;
- use an accessible progressbar where determinate;
- not announce updates so frequently that assistive technology becomes unusable;
- expose cancellation when supported;
- announce completion and errors clearly.

## Motion

Respect `prefers-reduced-motion` for non-essential animation. Do not use continuous blinking, rapid pulsing or layout motion to communicate required information.

## Text and layout

- Support Windows display scaling and browser text scaling.
- Avoid fixed-height containers that clip enlarged text.
- Keep minimum window dimensions practical without preventing resize.
- Avoid dense paragraphs inside destructive confirmation controls.
- Use headings and landmarks to separate scan, findings, plan and restore workflows.
- Keep error text selectable and readable.

## Forms and errors

- Associate labels with controls.
- Explain required fields before submission.
- Place error text near the relevant control when possible.
- Do not clear user input after a recoverable error.
- Use plain language and preserve useful technical detail for diagnostics.

## Testing checklist

For visible changes, test:

1. keyboard navigation from launch through a Quick scan;
2. finding selection and plan review without a mouse;
3. visible focus on every interactive element;
4. 200% text scaling;
5. Windows high-contrast or forced-colour behaviour where practical;
6. reduced-motion mode;
7. screen-reader names for icons and progress;
8. consequence comprehension without colour;
9. restore and updater error announcements.

Document known limitations in the pull request.

## Contribution requirement

A feature is incomplete when its only usable path requires precise pointer interaction, hover-only content or colour-only interpretation. Accessibility regressions should be treated as product defects, especially when they hide safety consequences.
