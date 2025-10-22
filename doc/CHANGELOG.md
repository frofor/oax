# Changelog

## 0.2.0

Breaking changes:
- Replaced CLI arguments with interactive prompts:
  - Replaced `--api-url` and `--spec-url` options with interactive URL prompt.
  - Replaced RPC selection with searchable select prompt.
  - Replaced `--param` option with type-aware interactive parameter configuration menu.
  - Parameters can be set, updated and removed via interactive menu.
  - Required parameters are now automatically initialized with default values based on their type.

## 0.1.0

Initial release:
- Added `get` method for executing GET requests:
  - Executes the request to the specified endpoint.
  - Supports API server URL and specification URL passing via `--api-url` and `--spec-url` options.
  - Supports parameter passing via `--param` option.
- Endpoint and parameter auto-completion based on the specification.
