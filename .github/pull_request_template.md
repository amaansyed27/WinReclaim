## Summary

Describe the user or developer problem and the implemented change.

## Change type

- [ ] Bug fix
- [ ] New detection rule
- [ ] New or changed cleanup adapter
- [ ] UI or accessibility
- [ ] Storage Assistant or optional OpenAI feature
- [ ] Installer, updater or CI
- [ ] Refactor
- [ ] Documentation

## Safety review

- [ ] The frontend does not supply arbitrary cleanup paths or commands.
- [ ] Protected-path precedence remains intact.
- [ ] Reparse points, links and stale filesystem state are handled safely.
- [ ] Recovery consequences are accurate and visible.
- [ ] Estimates are not presented as measured results.
- [ ] Any filesystem mutation has refusal-path tests.
- [ ] Optional AI output remains advisory and cannot enable or execute cleanup.
- [ ] New network access or persisted data is documented.
- [ ] No credentials, private paths or signing keys are included.

Explain any item that is not applicable or requires special review:

## Testing

Commands run:

```text
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Manual scenarios tested:

- 

Not tested or remaining limitations:

- 

## Documentation and compatibility

- [ ] Relevant README/docs were updated.
- [ ] `CHANGELOG.md` was updated under **Unreleased**.
- [ ] Persisted schema compatibility was considered.
- [ ] Release or updater implications were considered.
- [ ] Third-party licensing/provenance was updated when applicable.

## Screenshots

Include sanitized screenshots for visible UI changes. Remove usernames, drive labels, project names, keys and private paths.
