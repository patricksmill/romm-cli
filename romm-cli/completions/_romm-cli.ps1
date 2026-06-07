
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'romm-cli' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'romm-cli'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'romm-cli' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Create or update user configuration')
            [CompletionResult]::new('setup', 'setup', [CompletionResultType]::ParameterValue, 'Create or update user configuration')
            [CompletionResult]::new('api', 'api', [CompletionResultType]::ParameterValue, 'Low-level access to any RomM API endpoint')
            [CompletionResult]::new('call', 'call', [CompletionResultType]::ParameterValue, 'Low-level access to any RomM API endpoint')
            [CompletionResult]::new('platforms', 'platforms', [CompletionResultType]::ParameterValue, 'Manage gaming platforms')
            [CompletionResult]::new('platform', 'platform', [CompletionResultType]::ParameterValue, 'Manage gaming platforms')
            [CompletionResult]::new('p', 'p', [CompletionResultType]::ParameterValue, 'Manage gaming platforms')
            [CompletionResult]::new('plats', 'plats', [CompletionResultType]::ParameterValue, 'Manage gaming platforms')
            [CompletionResult]::new('roms', 'roms', [CompletionResultType]::ParameterValue, 'Manage ROM files and metadata')
            [CompletionResult]::new('rom', 'rom', [CompletionResultType]::ParameterValue, 'Manage ROM files and metadata')
            [CompletionResult]::new('r', 'r', [CompletionResultType]::ParameterValue, 'Manage ROM files and metadata')
            [CompletionResult]::new('scan', 'scan', [CompletionResultType]::ParameterValue, 'Trigger a library scan on the RomM server')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Save-sync workflows (device registration, planning, and execution)')
            [CompletionResult]::new('download', 'download', [CompletionResultType]::ParameterValue, 'Download a ROM or related extras from the server')
            [CompletionResult]::new('dl', 'dl', [CompletionResultType]::ParameterValue, 'Download a ROM or related extras from the server')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Download a ROM or related extras from the server')
            [CompletionResult]::new('cache', 'cache', [CompletionResultType]::ParameterValue, 'Manage the local persistent cache')
            [CompletionResult]::new('auth', 'auth', [CompletionResultType]::ParameterValue, 'Manage authentication credentials')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Check for and install application updates')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;init' {
            [CompletionResult]::new('--url', '--url', [CompletionResultType]::ParameterName, 'RomM origin URL (e.g. <https://romm.example>). If provided with a token, skips interactive prompts')
            [CompletionResult]::new('--token', '--token', [CompletionResultType]::ParameterName, 'Bearer token string (discouraged: visible in process list)')
            [CompletionResult]::new('--token-file', '--token-file', [CompletionResultType]::ParameterName, 'Read Bearer token from a UTF-8 file. Use ''-'' for stdin')
            [CompletionResult]::new('--download-dir', '--download-dir', [CompletionResultType]::ParameterName, 'ROMs directory')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Overwrite existing user config `config.json` without asking')
            [CompletionResult]::new('--print-path', '--print-path', [CompletionResultType]::ParameterName, 'Print the path to the user config `config.json` and exit')
            [CompletionResult]::new('--no-https', '--no-https', [CompletionResultType]::ParameterName, 'Disable HTTPS (use HTTP instead)')
            [CompletionResult]::new('--check', '--check', [CompletionResultType]::ParameterName, 'Verify URL and token by fetching OpenAPI after saving')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;setup' {
            [CompletionResult]::new('--url', '--url', [CompletionResultType]::ParameterName, 'RomM origin URL (e.g. <https://romm.example>). If provided with a token, skips interactive prompts')
            [CompletionResult]::new('--token', '--token', [CompletionResultType]::ParameterName, 'Bearer token string (discouraged: visible in process list)')
            [CompletionResult]::new('--token-file', '--token-file', [CompletionResultType]::ParameterName, 'Read Bearer token from a UTF-8 file. Use ''-'' for stdin')
            [CompletionResult]::new('--download-dir', '--download-dir', [CompletionResultType]::ParameterName, 'ROMs directory')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Overwrite existing user config `config.json` without asking')
            [CompletionResult]::new('--print-path', '--print-path', [CompletionResultType]::ParameterName, 'Print the path to the user config `config.json` and exit')
            [CompletionResult]::new('--no-https', '--no-https', [CompletionResultType]::ParameterName, 'Disable HTTPS (use HTTP instead)')
            [CompletionResult]::new('--check', '--check', [CompletionResultType]::ParameterName, 'Verify URL and token by fetching OpenAPI after saving')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;api' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'Query parameters as key=value, repeatable')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'JSON request body as a string')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('call', 'call', [CompletionResultType]::ParameterValue, 'Make a generic API call')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Shortcut for GET request')
            [CompletionResult]::new('post', 'post', [CompletionResultType]::ParameterValue, 'Shortcut for POST request')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;call' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'Query parameters as key=value, repeatable')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'JSON request body as a string')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('call', 'call', [CompletionResultType]::ParameterValue, 'Make a generic API call')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Shortcut for GET request')
            [CompletionResult]::new('post', 'post', [CompletionResultType]::ParameterValue, 'Shortcut for POST request')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;api;call' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'Query parameters as key=value, repeatable')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'JSON request body as a string')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;call;call' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'Query parameters as key=value, repeatable')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'JSON request body as a string')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;api;get' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'Query parameters as key=value, repeatable')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'JSON request body as a string')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;call;get' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'Query parameters as key=value, repeatable')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'JSON request body as a string')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;api;post' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'Query parameters as key=value, repeatable')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'JSON request body as a string')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;call;post' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'Query parameters as key=value, repeatable')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'JSON request body as a string')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;api;help' {
            [CompletionResult]::new('call', 'call', [CompletionResultType]::ParameterValue, 'Make a generic API call')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Shortcut for GET request')
            [CompletionResult]::new('post', 'post', [CompletionResultType]::ParameterValue, 'Shortcut for POST request')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;api;help;call' {
            break
        }
        'romm-cli;api;help;get' {
            break
        }
        'romm-cli;api;help;post' {
            break
        }
        'romm-cli;api;help;help' {
            break
        }
        'romm-cli;call;help' {
            [CompletionResult]::new('call', 'call', [CompletionResultType]::ParameterValue, 'Make a generic API call')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Shortcut for GET request')
            [CompletionResult]::new('post', 'post', [CompletionResultType]::ParameterValue, 'Shortcut for POST request')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;call;help;call' {
            break
        }
        'romm-cli;call;help;get' {
            break
        }
        'romm-cli;call;help;post' {
            break
        }
        'romm-cli;call;help;help' {
            break
        }
        'romm-cli;platforms' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('ls', 'ls', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;platform' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('ls', 'ls', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;p' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('ls', 'ls', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;plats' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('ls', 'ls', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;platforms;list' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;platforms;ls' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;platform;list' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;platform;ls' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;p;list' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;p;ls' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;plats;list' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;plats;ls' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;platforms;get' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;platforms;info' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;platform;get' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;platform;info' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;p;get' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;p;info' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;plats;get' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;plats;info' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;platforms;help' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;platforms;help;list' {
            break
        }
        'romm-cli;platforms;help;get' {
            break
        }
        'romm-cli;platforms;help;help' {
            break
        }
        'romm-cli;platform;help' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;platform;help;list' {
            break
        }
        'romm-cli;platform;help;get' {
            break
        }
        'romm-cli;platform;help;help' {
            break
        }
        'romm-cli;p;help' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;p;help;list' {
            break
        }
        'romm-cli;p;help;get' {
            break
        }
        'romm-cli;p;help;help' {
            break
        }
        'romm-cli;plats;help' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;plats;help;list' {
            break
        }
        'romm-cli;plats;help;get' {
            break
        }
        'romm-cli;plats;help;help' {
            break
        }
        'romm-cli;roms' {
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'search-term')
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'search-term')
            [CompletionResult]::new('--q', '--q', [CompletionResultType]::ParameterName, 'search-term')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Platform slug or name; repeat for multiple `platform_ids`')
            [CompletionResult]::new('--p', '--p', [CompletionResultType]::ParameterName, 'Platform slug or name; repeat for multiple `platform_ids`')
            [CompletionResult]::new('--collection', '--collection', [CompletionResultType]::ParameterName, 'Manual collection id or exact name')
            [CompletionResult]::new('--smart-collection', '--smart-collection', [CompletionResultType]::ParameterName, 'Smart collection id or exact name')
            [CompletionResult]::new('--virtual-collection', '--virtual-collection', [CompletionResultType]::ParameterName, 'Virtual collection id (e.g. recent)')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'limit')
            [CompletionResult]::new('--offset', '--offset', [CompletionResultType]::ParameterName, 'offset')
            [CompletionResult]::new('--matched', '--matched', [CompletionResultType]::ParameterName, 'matched')
            [CompletionResult]::new('--favorite', '--favorite', [CompletionResultType]::ParameterName, 'favorite')
            [CompletionResult]::new('--duplicate', '--duplicate', [CompletionResultType]::ParameterName, 'duplicate')
            [CompletionResult]::new('--last-played', '--last-played', [CompletionResultType]::ParameterName, 'last-played')
            [CompletionResult]::new('--playable', '--playable', [CompletionResultType]::ParameterName, 'playable')
            [CompletionResult]::new('--missing', '--missing', [CompletionResultType]::ParameterName, 'missing')
            [CompletionResult]::new('--has-ra', '--has-ra', [CompletionResultType]::ParameterName, 'has-ra')
            [CompletionResult]::new('--verified', '--verified', [CompletionResultType]::ParameterName, 'verified')
            [CompletionResult]::new('--group-by-meta-id', '--group-by-meta-id', [CompletionResultType]::ParameterName, 'group-by-meta-id')
            [CompletionResult]::new('--with-char-index', '--with-char-index', [CompletionResultType]::ParameterName, 'with-char-index')
            [CompletionResult]::new('--with-filter-values', '--with-filter-values', [CompletionResultType]::ParameterName, 'with-filter-values')
            [CompletionResult]::new('--genre', '--genre', [CompletionResultType]::ParameterName, 'genre')
            [CompletionResult]::new('--franchise', '--franchise', [CompletionResultType]::ParameterName, 'franchise')
            [CompletionResult]::new('--collection-tag', '--collection-tag', [CompletionResultType]::ParameterName, 'collection-tag')
            [CompletionResult]::new('--company', '--company', [CompletionResultType]::ParameterName, 'company')
            [CompletionResult]::new('--age-rating', '--age-rating', [CompletionResultType]::ParameterName, 'age-rating')
            [CompletionResult]::new('--status', '--status', [CompletionResultType]::ParameterName, 'status')
            [CompletionResult]::new('--region', '--region', [CompletionResultType]::ParameterName, 'region')
            [CompletionResult]::new('--language', '--language', [CompletionResultType]::ParameterName, 'language')
            [CompletionResult]::new('--player-count', '--player-count', [CompletionResultType]::ParameterName, 'player-count')
            [CompletionResult]::new('--genres-logic', '--genres-logic', [CompletionResultType]::ParameterName, 'genres-logic')
            [CompletionResult]::new('--franchises-logic', '--franchises-logic', [CompletionResultType]::ParameterName, 'franchises-logic')
            [CompletionResult]::new('--collections-logic', '--collections-logic', [CompletionResultType]::ParameterName, 'collections-logic')
            [CompletionResult]::new('--companies-logic', '--companies-logic', [CompletionResultType]::ParameterName, 'companies-logic')
            [CompletionResult]::new('--age-ratings-logic', '--age-ratings-logic', [CompletionResultType]::ParameterName, 'age-ratings-logic')
            [CompletionResult]::new('--regions-logic', '--regions-logic', [CompletionResultType]::ParameterName, 'regions-logic')
            [CompletionResult]::new('--languages-logic', '--languages-logic', [CompletionResultType]::ParameterName, 'languages-logic')
            [CompletionResult]::new('--statuses-logic', '--statuses-logic', [CompletionResultType]::ParameterName, 'statuses-logic')
            [CompletionResult]::new('--player-counts-logic', '--player-counts-logic', [CompletionResultType]::ParameterName, 'player-counts-logic')
            [CompletionResult]::new('--order-by', '--order-by', [CompletionResultType]::ParameterName, 'order-by')
            [CompletionResult]::new('--order-dir', '--order-dir', [CompletionResultType]::ParameterName, 'order-dir')
            [CompletionResult]::new('--updated-after', '--updated-after', [CompletionResultType]::ParameterName, 'updated-after')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get detailed information for a single ROM')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Get detailed information for a single ROM')
            [CompletionResult]::new('find', 'find', [CompletionResultType]::ParameterValue, 'Lookup ROM by file hash or metadata provider id')
            [CompletionResult]::new('filters', 'filters', [CompletionResultType]::ParameterValue, 'Print canonical filter values from `GET /api/roms/filters`')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete ROMs from the database (optional filesystem delete)')
            [CompletionResult]::new('props', 'props', [CompletionResultType]::ParameterValue, 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)')
            [CompletionResult]::new('notes-list', 'notes-list', [CompletionResultType]::ParameterValue, 'List notes for a ROM')
            [CompletionResult]::new('notes-add', 'notes-add', [CompletionResultType]::ParameterValue, 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})')
            [CompletionResult]::new('notes-update', 'notes-update', [CompletionResultType]::ParameterValue, 'Update a note')
            [CompletionResult]::new('notes-delete', 'notes-delete', [CompletionResultType]::ParameterValue, 'Delete a note')
            [CompletionResult]::new('manuals-add', 'manuals-add', [CompletionResultType]::ParameterValue, 'Upload a manual file (`POST /api/roms/{id}/manuals`)')
            [CompletionResult]::new('cover-search', 'cover-search', [CompletionResultType]::ParameterValue, 'Search covers and metadata matches')
            [CompletionResult]::new('upload', 'upload', [CompletionResultType]::ParameterValue, 'Upload a ROM file to a platform')
            [CompletionResult]::new('up', 'up', [CompletionResultType]::ParameterValue, 'Upload a ROM file to a platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;rom' {
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'search-term')
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'search-term')
            [CompletionResult]::new('--q', '--q', [CompletionResultType]::ParameterName, 'search-term')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Platform slug or name; repeat for multiple `platform_ids`')
            [CompletionResult]::new('--p', '--p', [CompletionResultType]::ParameterName, 'Platform slug or name; repeat for multiple `platform_ids`')
            [CompletionResult]::new('--collection', '--collection', [CompletionResultType]::ParameterName, 'Manual collection id or exact name')
            [CompletionResult]::new('--smart-collection', '--smart-collection', [CompletionResultType]::ParameterName, 'Smart collection id or exact name')
            [CompletionResult]::new('--virtual-collection', '--virtual-collection', [CompletionResultType]::ParameterName, 'Virtual collection id (e.g. recent)')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'limit')
            [CompletionResult]::new('--offset', '--offset', [CompletionResultType]::ParameterName, 'offset')
            [CompletionResult]::new('--matched', '--matched', [CompletionResultType]::ParameterName, 'matched')
            [CompletionResult]::new('--favorite', '--favorite', [CompletionResultType]::ParameterName, 'favorite')
            [CompletionResult]::new('--duplicate', '--duplicate', [CompletionResultType]::ParameterName, 'duplicate')
            [CompletionResult]::new('--last-played', '--last-played', [CompletionResultType]::ParameterName, 'last-played')
            [CompletionResult]::new('--playable', '--playable', [CompletionResultType]::ParameterName, 'playable')
            [CompletionResult]::new('--missing', '--missing', [CompletionResultType]::ParameterName, 'missing')
            [CompletionResult]::new('--has-ra', '--has-ra', [CompletionResultType]::ParameterName, 'has-ra')
            [CompletionResult]::new('--verified', '--verified', [CompletionResultType]::ParameterName, 'verified')
            [CompletionResult]::new('--group-by-meta-id', '--group-by-meta-id', [CompletionResultType]::ParameterName, 'group-by-meta-id')
            [CompletionResult]::new('--with-char-index', '--with-char-index', [CompletionResultType]::ParameterName, 'with-char-index')
            [CompletionResult]::new('--with-filter-values', '--with-filter-values', [CompletionResultType]::ParameterName, 'with-filter-values')
            [CompletionResult]::new('--genre', '--genre', [CompletionResultType]::ParameterName, 'genre')
            [CompletionResult]::new('--franchise', '--franchise', [CompletionResultType]::ParameterName, 'franchise')
            [CompletionResult]::new('--collection-tag', '--collection-tag', [CompletionResultType]::ParameterName, 'collection-tag')
            [CompletionResult]::new('--company', '--company', [CompletionResultType]::ParameterName, 'company')
            [CompletionResult]::new('--age-rating', '--age-rating', [CompletionResultType]::ParameterName, 'age-rating')
            [CompletionResult]::new('--status', '--status', [CompletionResultType]::ParameterName, 'status')
            [CompletionResult]::new('--region', '--region', [CompletionResultType]::ParameterName, 'region')
            [CompletionResult]::new('--language', '--language', [CompletionResultType]::ParameterName, 'language')
            [CompletionResult]::new('--player-count', '--player-count', [CompletionResultType]::ParameterName, 'player-count')
            [CompletionResult]::new('--genres-logic', '--genres-logic', [CompletionResultType]::ParameterName, 'genres-logic')
            [CompletionResult]::new('--franchises-logic', '--franchises-logic', [CompletionResultType]::ParameterName, 'franchises-logic')
            [CompletionResult]::new('--collections-logic', '--collections-logic', [CompletionResultType]::ParameterName, 'collections-logic')
            [CompletionResult]::new('--companies-logic', '--companies-logic', [CompletionResultType]::ParameterName, 'companies-logic')
            [CompletionResult]::new('--age-ratings-logic', '--age-ratings-logic', [CompletionResultType]::ParameterName, 'age-ratings-logic')
            [CompletionResult]::new('--regions-logic', '--regions-logic', [CompletionResultType]::ParameterName, 'regions-logic')
            [CompletionResult]::new('--languages-logic', '--languages-logic', [CompletionResultType]::ParameterName, 'languages-logic')
            [CompletionResult]::new('--statuses-logic', '--statuses-logic', [CompletionResultType]::ParameterName, 'statuses-logic')
            [CompletionResult]::new('--player-counts-logic', '--player-counts-logic', [CompletionResultType]::ParameterName, 'player-counts-logic')
            [CompletionResult]::new('--order-by', '--order-by', [CompletionResultType]::ParameterName, 'order-by')
            [CompletionResult]::new('--order-dir', '--order-dir', [CompletionResultType]::ParameterName, 'order-dir')
            [CompletionResult]::new('--updated-after', '--updated-after', [CompletionResultType]::ParameterName, 'updated-after')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get detailed information for a single ROM')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Get detailed information for a single ROM')
            [CompletionResult]::new('find', 'find', [CompletionResultType]::ParameterValue, 'Lookup ROM by file hash or metadata provider id')
            [CompletionResult]::new('filters', 'filters', [CompletionResultType]::ParameterValue, 'Print canonical filter values from `GET /api/roms/filters`')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete ROMs from the database (optional filesystem delete)')
            [CompletionResult]::new('props', 'props', [CompletionResultType]::ParameterValue, 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)')
            [CompletionResult]::new('notes-list', 'notes-list', [CompletionResultType]::ParameterValue, 'List notes for a ROM')
            [CompletionResult]::new('notes-add', 'notes-add', [CompletionResultType]::ParameterValue, 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})')
            [CompletionResult]::new('notes-update', 'notes-update', [CompletionResultType]::ParameterValue, 'Update a note')
            [CompletionResult]::new('notes-delete', 'notes-delete', [CompletionResultType]::ParameterValue, 'Delete a note')
            [CompletionResult]::new('manuals-add', 'manuals-add', [CompletionResultType]::ParameterValue, 'Upload a manual file (`POST /api/roms/{id}/manuals`)')
            [CompletionResult]::new('cover-search', 'cover-search', [CompletionResultType]::ParameterValue, 'Search covers and metadata matches')
            [CompletionResult]::new('upload', 'upload', [CompletionResultType]::ParameterValue, 'Upload a ROM file to a platform')
            [CompletionResult]::new('up', 'up', [CompletionResultType]::ParameterValue, 'Upload a ROM file to a platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;r' {
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'search-term')
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'search-term')
            [CompletionResult]::new('--q', '--q', [CompletionResultType]::ParameterName, 'search-term')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Platform slug or name; repeat for multiple `platform_ids`')
            [CompletionResult]::new('--p', '--p', [CompletionResultType]::ParameterName, 'Platform slug or name; repeat for multiple `platform_ids`')
            [CompletionResult]::new('--collection', '--collection', [CompletionResultType]::ParameterName, 'Manual collection id or exact name')
            [CompletionResult]::new('--smart-collection', '--smart-collection', [CompletionResultType]::ParameterName, 'Smart collection id or exact name')
            [CompletionResult]::new('--virtual-collection', '--virtual-collection', [CompletionResultType]::ParameterName, 'Virtual collection id (e.g. recent)')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'limit')
            [CompletionResult]::new('--offset', '--offset', [CompletionResultType]::ParameterName, 'offset')
            [CompletionResult]::new('--matched', '--matched', [CompletionResultType]::ParameterName, 'matched')
            [CompletionResult]::new('--favorite', '--favorite', [CompletionResultType]::ParameterName, 'favorite')
            [CompletionResult]::new('--duplicate', '--duplicate', [CompletionResultType]::ParameterName, 'duplicate')
            [CompletionResult]::new('--last-played', '--last-played', [CompletionResultType]::ParameterName, 'last-played')
            [CompletionResult]::new('--playable', '--playable', [CompletionResultType]::ParameterName, 'playable')
            [CompletionResult]::new('--missing', '--missing', [CompletionResultType]::ParameterName, 'missing')
            [CompletionResult]::new('--has-ra', '--has-ra', [CompletionResultType]::ParameterName, 'has-ra')
            [CompletionResult]::new('--verified', '--verified', [CompletionResultType]::ParameterName, 'verified')
            [CompletionResult]::new('--group-by-meta-id', '--group-by-meta-id', [CompletionResultType]::ParameterName, 'group-by-meta-id')
            [CompletionResult]::new('--with-char-index', '--with-char-index', [CompletionResultType]::ParameterName, 'with-char-index')
            [CompletionResult]::new('--with-filter-values', '--with-filter-values', [CompletionResultType]::ParameterName, 'with-filter-values')
            [CompletionResult]::new('--genre', '--genre', [CompletionResultType]::ParameterName, 'genre')
            [CompletionResult]::new('--franchise', '--franchise', [CompletionResultType]::ParameterName, 'franchise')
            [CompletionResult]::new('--collection-tag', '--collection-tag', [CompletionResultType]::ParameterName, 'collection-tag')
            [CompletionResult]::new('--company', '--company', [CompletionResultType]::ParameterName, 'company')
            [CompletionResult]::new('--age-rating', '--age-rating', [CompletionResultType]::ParameterName, 'age-rating')
            [CompletionResult]::new('--status', '--status', [CompletionResultType]::ParameterName, 'status')
            [CompletionResult]::new('--region', '--region', [CompletionResultType]::ParameterName, 'region')
            [CompletionResult]::new('--language', '--language', [CompletionResultType]::ParameterName, 'language')
            [CompletionResult]::new('--player-count', '--player-count', [CompletionResultType]::ParameterName, 'player-count')
            [CompletionResult]::new('--genres-logic', '--genres-logic', [CompletionResultType]::ParameterName, 'genres-logic')
            [CompletionResult]::new('--franchises-logic', '--franchises-logic', [CompletionResultType]::ParameterName, 'franchises-logic')
            [CompletionResult]::new('--collections-logic', '--collections-logic', [CompletionResultType]::ParameterName, 'collections-logic')
            [CompletionResult]::new('--companies-logic', '--companies-logic', [CompletionResultType]::ParameterName, 'companies-logic')
            [CompletionResult]::new('--age-ratings-logic', '--age-ratings-logic', [CompletionResultType]::ParameterName, 'age-ratings-logic')
            [CompletionResult]::new('--regions-logic', '--regions-logic', [CompletionResultType]::ParameterName, 'regions-logic')
            [CompletionResult]::new('--languages-logic', '--languages-logic', [CompletionResultType]::ParameterName, 'languages-logic')
            [CompletionResult]::new('--statuses-logic', '--statuses-logic', [CompletionResultType]::ParameterName, 'statuses-logic')
            [CompletionResult]::new('--player-counts-logic', '--player-counts-logic', [CompletionResultType]::ParameterName, 'player-counts-logic')
            [CompletionResult]::new('--order-by', '--order-by', [CompletionResultType]::ParameterName, 'order-by')
            [CompletionResult]::new('--order-dir', '--order-dir', [CompletionResultType]::ParameterName, 'order-dir')
            [CompletionResult]::new('--updated-after', '--updated-after', [CompletionResultType]::ParameterName, 'updated-after')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get detailed information for a single ROM')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Get detailed information for a single ROM')
            [CompletionResult]::new('find', 'find', [CompletionResultType]::ParameterValue, 'Lookup ROM by file hash or metadata provider id')
            [CompletionResult]::new('filters', 'filters', [CompletionResultType]::ParameterValue, 'Print canonical filter values from `GET /api/roms/filters`')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete ROMs from the database (optional filesystem delete)')
            [CompletionResult]::new('props', 'props', [CompletionResultType]::ParameterValue, 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)')
            [CompletionResult]::new('notes-list', 'notes-list', [CompletionResultType]::ParameterValue, 'List notes for a ROM')
            [CompletionResult]::new('notes-add', 'notes-add', [CompletionResultType]::ParameterValue, 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})')
            [CompletionResult]::new('notes-update', 'notes-update', [CompletionResultType]::ParameterValue, 'Update a note')
            [CompletionResult]::new('notes-delete', 'notes-delete', [CompletionResultType]::ParameterValue, 'Delete a note')
            [CompletionResult]::new('manuals-add', 'manuals-add', [CompletionResultType]::ParameterValue, 'Upload a manual file (`POST /api/roms/{id}/manuals`)')
            [CompletionResult]::new('cover-search', 'cover-search', [CompletionResultType]::ParameterValue, 'Search covers and metadata matches')
            [CompletionResult]::new('upload', 'upload', [CompletionResultType]::ParameterValue, 'Upload a ROM file to a platform')
            [CompletionResult]::new('up', 'up', [CompletionResultType]::ParameterValue, 'Upload a ROM file to a platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;roms;get' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;info' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;get' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;info' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;get' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;info' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;find' {
            [CompletionResult]::new('--crc', '--crc', [CompletionResultType]::ParameterName, 'crc')
            [CompletionResult]::new('--md5', '--md5', [CompletionResultType]::ParameterName, 'md5')
            [CompletionResult]::new('--sha1', '--sha1', [CompletionResultType]::ParameterName, 'sha1')
            [CompletionResult]::new('--igdb-id', '--igdb-id', [CompletionResultType]::ParameterName, 'igdb-id')
            [CompletionResult]::new('--moby-id', '--moby-id', [CompletionResultType]::ParameterName, 'moby-id')
            [CompletionResult]::new('--ss-id', '--ss-id', [CompletionResultType]::ParameterName, 'ss-id')
            [CompletionResult]::new('--ra-id', '--ra-id', [CompletionResultType]::ParameterName, 'ra-id')
            [CompletionResult]::new('--launchbox-id', '--launchbox-id', [CompletionResultType]::ParameterName, 'launchbox-id')
            [CompletionResult]::new('--hasheous-id', '--hasheous-id', [CompletionResultType]::ParameterName, 'hasheous-id')
            [CompletionResult]::new('--tgdb-id', '--tgdb-id', [CompletionResultType]::ParameterName, 'tgdb-id')
            [CompletionResult]::new('--flashpoint-id', '--flashpoint-id', [CompletionResultType]::ParameterName, 'flashpoint-id')
            [CompletionResult]::new('--hltb-id', '--hltb-id', [CompletionResultType]::ParameterName, 'hltb-id')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;find' {
            [CompletionResult]::new('--crc', '--crc', [CompletionResultType]::ParameterName, 'crc')
            [CompletionResult]::new('--md5', '--md5', [CompletionResultType]::ParameterName, 'md5')
            [CompletionResult]::new('--sha1', '--sha1', [CompletionResultType]::ParameterName, 'sha1')
            [CompletionResult]::new('--igdb-id', '--igdb-id', [CompletionResultType]::ParameterName, 'igdb-id')
            [CompletionResult]::new('--moby-id', '--moby-id', [CompletionResultType]::ParameterName, 'moby-id')
            [CompletionResult]::new('--ss-id', '--ss-id', [CompletionResultType]::ParameterName, 'ss-id')
            [CompletionResult]::new('--ra-id', '--ra-id', [CompletionResultType]::ParameterName, 'ra-id')
            [CompletionResult]::new('--launchbox-id', '--launchbox-id', [CompletionResultType]::ParameterName, 'launchbox-id')
            [CompletionResult]::new('--hasheous-id', '--hasheous-id', [CompletionResultType]::ParameterName, 'hasheous-id')
            [CompletionResult]::new('--tgdb-id', '--tgdb-id', [CompletionResultType]::ParameterName, 'tgdb-id')
            [CompletionResult]::new('--flashpoint-id', '--flashpoint-id', [CompletionResultType]::ParameterName, 'flashpoint-id')
            [CompletionResult]::new('--hltb-id', '--hltb-id', [CompletionResultType]::ParameterName, 'hltb-id')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;find' {
            [CompletionResult]::new('--crc', '--crc', [CompletionResultType]::ParameterName, 'crc')
            [CompletionResult]::new('--md5', '--md5', [CompletionResultType]::ParameterName, 'md5')
            [CompletionResult]::new('--sha1', '--sha1', [CompletionResultType]::ParameterName, 'sha1')
            [CompletionResult]::new('--igdb-id', '--igdb-id', [CompletionResultType]::ParameterName, 'igdb-id')
            [CompletionResult]::new('--moby-id', '--moby-id', [CompletionResultType]::ParameterName, 'moby-id')
            [CompletionResult]::new('--ss-id', '--ss-id', [CompletionResultType]::ParameterName, 'ss-id')
            [CompletionResult]::new('--ra-id', '--ra-id', [CompletionResultType]::ParameterName, 'ra-id')
            [CompletionResult]::new('--launchbox-id', '--launchbox-id', [CompletionResultType]::ParameterName, 'launchbox-id')
            [CompletionResult]::new('--hasheous-id', '--hasheous-id', [CompletionResultType]::ParameterName, 'hasheous-id')
            [CompletionResult]::new('--tgdb-id', '--tgdb-id', [CompletionResultType]::ParameterName, 'tgdb-id')
            [CompletionResult]::new('--flashpoint-id', '--flashpoint-id', [CompletionResultType]::ParameterName, 'flashpoint-id')
            [CompletionResult]::new('--hltb-id', '--hltb-id', [CompletionResultType]::ParameterName, 'hltb-id')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;filters' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;filters' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;filters' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;delete' {
            [CompletionResult]::new('--delete-from-fs', '--delete-from-fs', [CompletionResultType]::ParameterName, 'Also delete these ROM ids from disk (repeat ids as needed)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Skip confirmation')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;delete' {
            [CompletionResult]::new('--delete-from-fs', '--delete-from-fs', [CompletionResultType]::ParameterName, 'Also delete these ROM ids from disk (repeat ids as needed)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Skip confirmation')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;delete' {
            [CompletionResult]::new('--delete-from-fs', '--delete-from-fs', [CompletionResultType]::ParameterName, 'Also delete these ROM ids from disk (repeat ids as needed)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Skip confirmation')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;props' {
            [CompletionResult]::new('--is-main-sibling', '--is-main-sibling', [CompletionResultType]::ParameterName, 'is-main-sibling')
            [CompletionResult]::new('--backlogged', '--backlogged', [CompletionResultType]::ParameterName, 'backlogged')
            [CompletionResult]::new('--now-playing', '--now-playing', [CompletionResultType]::ParameterName, 'now-playing')
            [CompletionResult]::new('--hidden', '--hidden', [CompletionResultType]::ParameterName, 'hidden')
            [CompletionResult]::new('--rating', '--rating', [CompletionResultType]::ParameterName, 'rating')
            [CompletionResult]::new('--difficulty', '--difficulty', [CompletionResultType]::ParameterName, 'difficulty')
            [CompletionResult]::new('--completion', '--completion', [CompletionResultType]::ParameterName, 'completion')
            [CompletionResult]::new('--status', '--status', [CompletionResultType]::ParameterName, 'status')
            [CompletionResult]::new('--update-last-played', '--update-last-played', [CompletionResultType]::ParameterName, 'update-last-played')
            [CompletionResult]::new('--remove-last-played', '--remove-last-played', [CompletionResultType]::ParameterName, 'remove-last-played')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;props' {
            [CompletionResult]::new('--is-main-sibling', '--is-main-sibling', [CompletionResultType]::ParameterName, 'is-main-sibling')
            [CompletionResult]::new('--backlogged', '--backlogged', [CompletionResultType]::ParameterName, 'backlogged')
            [CompletionResult]::new('--now-playing', '--now-playing', [CompletionResultType]::ParameterName, 'now-playing')
            [CompletionResult]::new('--hidden', '--hidden', [CompletionResultType]::ParameterName, 'hidden')
            [CompletionResult]::new('--rating', '--rating', [CompletionResultType]::ParameterName, 'rating')
            [CompletionResult]::new('--difficulty', '--difficulty', [CompletionResultType]::ParameterName, 'difficulty')
            [CompletionResult]::new('--completion', '--completion', [CompletionResultType]::ParameterName, 'completion')
            [CompletionResult]::new('--status', '--status', [CompletionResultType]::ParameterName, 'status')
            [CompletionResult]::new('--update-last-played', '--update-last-played', [CompletionResultType]::ParameterName, 'update-last-played')
            [CompletionResult]::new('--remove-last-played', '--remove-last-played', [CompletionResultType]::ParameterName, 'remove-last-played')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;props' {
            [CompletionResult]::new('--is-main-sibling', '--is-main-sibling', [CompletionResultType]::ParameterName, 'is-main-sibling')
            [CompletionResult]::new('--backlogged', '--backlogged', [CompletionResultType]::ParameterName, 'backlogged')
            [CompletionResult]::new('--now-playing', '--now-playing', [CompletionResultType]::ParameterName, 'now-playing')
            [CompletionResult]::new('--hidden', '--hidden', [CompletionResultType]::ParameterName, 'hidden')
            [CompletionResult]::new('--rating', '--rating', [CompletionResultType]::ParameterName, 'rating')
            [CompletionResult]::new('--difficulty', '--difficulty', [CompletionResultType]::ParameterName, 'difficulty')
            [CompletionResult]::new('--completion', '--completion', [CompletionResultType]::ParameterName, 'completion')
            [CompletionResult]::new('--status', '--status', [CompletionResultType]::ParameterName, 'status')
            [CompletionResult]::new('--update-last-played', '--update-last-played', [CompletionResultType]::ParameterName, 'update-last-played')
            [CompletionResult]::new('--remove-last-played', '--remove-last-played', [CompletionResultType]::ParameterName, 'remove-last-played')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;notes-list' {
            [CompletionResult]::new('--public-only', '--public-only', [CompletionResultType]::ParameterName, 'public-only')
            [CompletionResult]::new('--search', '--search', [CompletionResultType]::ParameterName, 'search')
            [CompletionResult]::new('--tag', '--tag', [CompletionResultType]::ParameterName, 'tag')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;notes-list' {
            [CompletionResult]::new('--public-only', '--public-only', [CompletionResultType]::ParameterName, 'public-only')
            [CompletionResult]::new('--search', '--search', [CompletionResultType]::ParameterName, 'search')
            [CompletionResult]::new('--tag', '--tag', [CompletionResultType]::ParameterName, 'tag')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;notes-list' {
            [CompletionResult]::new('--public-only', '--public-only', [CompletionResultType]::ParameterName, 'public-only')
            [CompletionResult]::new('--search', '--search', [CompletionResultType]::ParameterName, 'search')
            [CompletionResult]::new('--tag', '--tag', [CompletionResultType]::ParameterName, 'tag')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;notes-add' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'JSON object')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;notes-add' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'JSON object')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;notes-add' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'JSON object')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;notes-update' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;notes-update' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;notes-update' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;notes-delete' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;notes-delete' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;notes-delete' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;manuals-add' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;manuals-add' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;manuals-add' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;cover-search' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'query')
            [CompletionResult]::new('--search-by', '--search-by', [CompletionResultType]::ParameterName, 'search-by')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;cover-search' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'query')
            [CompletionResult]::new('--search-by', '--search-by', [CompletionResultType]::ParameterName, 'search-by')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;cover-search' {
            [CompletionResult]::new('--query', '--query', [CompletionResultType]::ParameterName, 'query')
            [CompletionResult]::new('--search-by', '--search-by', [CompletionResultType]::ParameterName, 'search-by')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;upload' {
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")')
            [CompletionResult]::new('--wait-timeout-secs', '--wait-timeout-secs', [CompletionResultType]::ParameterName, 'Max seconds to wait when `--wait` is set (default: 3600)')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--scan', '--scan', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--wait', '--wait', [CompletionResultType]::ParameterName, 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;up' {
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")')
            [CompletionResult]::new('--wait-timeout-secs', '--wait-timeout-secs', [CompletionResultType]::ParameterName, 'Max seconds to wait when `--wait` is set (default: 3600)')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--scan', '--scan', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--wait', '--wait', [CompletionResultType]::ParameterName, 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;upload' {
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")')
            [CompletionResult]::new('--wait-timeout-secs', '--wait-timeout-secs', [CompletionResultType]::ParameterName, 'Max seconds to wait when `--wait` is set (default: 3600)')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--scan', '--scan', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--wait', '--wait', [CompletionResultType]::ParameterName, 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;rom;up' {
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")')
            [CompletionResult]::new('--wait-timeout-secs', '--wait-timeout-secs', [CompletionResultType]::ParameterName, 'Max seconds to wait when `--wait` is set (default: 3600)')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--scan', '--scan', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--wait', '--wait', [CompletionResultType]::ParameterName, 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;upload' {
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")')
            [CompletionResult]::new('--wait-timeout-secs', '--wait-timeout-secs', [CompletionResultType]::ParameterName, 'Max seconds to wait when `--wait` is set (default: 3600)')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--scan', '--scan', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--wait', '--wait', [CompletionResultType]::ParameterName, 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;r;up' {
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")')
            [CompletionResult]::new('--wait-timeout-secs', '--wait-timeout-secs', [CompletionResultType]::ParameterName, 'Max seconds to wait when `--wait` is set (default: 3600)')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--scan', '--scan', [CompletionResultType]::ParameterName, 'Trigger a library scan after upload completes')
            [CompletionResult]::new('--wait', '--wait', [CompletionResultType]::ParameterName, 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;roms;help' {
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get detailed information for a single ROM')
            [CompletionResult]::new('find', 'find', [CompletionResultType]::ParameterValue, 'Lookup ROM by file hash or metadata provider id')
            [CompletionResult]::new('filters', 'filters', [CompletionResultType]::ParameterValue, 'Print canonical filter values from `GET /api/roms/filters`')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete ROMs from the database (optional filesystem delete)')
            [CompletionResult]::new('props', 'props', [CompletionResultType]::ParameterValue, 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)')
            [CompletionResult]::new('notes-list', 'notes-list', [CompletionResultType]::ParameterValue, 'List notes for a ROM')
            [CompletionResult]::new('notes-add', 'notes-add', [CompletionResultType]::ParameterValue, 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})')
            [CompletionResult]::new('notes-update', 'notes-update', [CompletionResultType]::ParameterValue, 'Update a note')
            [CompletionResult]::new('notes-delete', 'notes-delete', [CompletionResultType]::ParameterValue, 'Delete a note')
            [CompletionResult]::new('manuals-add', 'manuals-add', [CompletionResultType]::ParameterValue, 'Upload a manual file (`POST /api/roms/{id}/manuals`)')
            [CompletionResult]::new('cover-search', 'cover-search', [CompletionResultType]::ParameterValue, 'Search covers and metadata matches')
            [CompletionResult]::new('upload', 'upload', [CompletionResultType]::ParameterValue, 'Upload a ROM file to a platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;roms;help;get' {
            break
        }
        'romm-cli;roms;help;find' {
            break
        }
        'romm-cli;roms;help;filters' {
            break
        }
        'romm-cli;roms;help;delete' {
            break
        }
        'romm-cli;roms;help;props' {
            break
        }
        'romm-cli;roms;help;notes-list' {
            break
        }
        'romm-cli;roms;help;notes-add' {
            break
        }
        'romm-cli;roms;help;notes-update' {
            break
        }
        'romm-cli;roms;help;notes-delete' {
            break
        }
        'romm-cli;roms;help;manuals-add' {
            break
        }
        'romm-cli;roms;help;cover-search' {
            break
        }
        'romm-cli;roms;help;upload' {
            break
        }
        'romm-cli;roms;help;help' {
            break
        }
        'romm-cli;rom;help' {
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get detailed information for a single ROM')
            [CompletionResult]::new('find', 'find', [CompletionResultType]::ParameterValue, 'Lookup ROM by file hash or metadata provider id')
            [CompletionResult]::new('filters', 'filters', [CompletionResultType]::ParameterValue, 'Print canonical filter values from `GET /api/roms/filters`')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete ROMs from the database (optional filesystem delete)')
            [CompletionResult]::new('props', 'props', [CompletionResultType]::ParameterValue, 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)')
            [CompletionResult]::new('notes-list', 'notes-list', [CompletionResultType]::ParameterValue, 'List notes for a ROM')
            [CompletionResult]::new('notes-add', 'notes-add', [CompletionResultType]::ParameterValue, 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})')
            [CompletionResult]::new('notes-update', 'notes-update', [CompletionResultType]::ParameterValue, 'Update a note')
            [CompletionResult]::new('notes-delete', 'notes-delete', [CompletionResultType]::ParameterValue, 'Delete a note')
            [CompletionResult]::new('manuals-add', 'manuals-add', [CompletionResultType]::ParameterValue, 'Upload a manual file (`POST /api/roms/{id}/manuals`)')
            [CompletionResult]::new('cover-search', 'cover-search', [CompletionResultType]::ParameterValue, 'Search covers and metadata matches')
            [CompletionResult]::new('upload', 'upload', [CompletionResultType]::ParameterValue, 'Upload a ROM file to a platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;rom;help;get' {
            break
        }
        'romm-cli;rom;help;find' {
            break
        }
        'romm-cli;rom;help;filters' {
            break
        }
        'romm-cli;rom;help;delete' {
            break
        }
        'romm-cli;rom;help;props' {
            break
        }
        'romm-cli;rom;help;notes-list' {
            break
        }
        'romm-cli;rom;help;notes-add' {
            break
        }
        'romm-cli;rom;help;notes-update' {
            break
        }
        'romm-cli;rom;help;notes-delete' {
            break
        }
        'romm-cli;rom;help;manuals-add' {
            break
        }
        'romm-cli;rom;help;cover-search' {
            break
        }
        'romm-cli;rom;help;upload' {
            break
        }
        'romm-cli;rom;help;help' {
            break
        }
        'romm-cli;r;help' {
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get detailed information for a single ROM')
            [CompletionResult]::new('find', 'find', [CompletionResultType]::ParameterValue, 'Lookup ROM by file hash or metadata provider id')
            [CompletionResult]::new('filters', 'filters', [CompletionResultType]::ParameterValue, 'Print canonical filter values from `GET /api/roms/filters`')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete ROMs from the database (optional filesystem delete)')
            [CompletionResult]::new('props', 'props', [CompletionResultType]::ParameterValue, 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)')
            [CompletionResult]::new('notes-list', 'notes-list', [CompletionResultType]::ParameterValue, 'List notes for a ROM')
            [CompletionResult]::new('notes-add', 'notes-add', [CompletionResultType]::ParameterValue, 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})')
            [CompletionResult]::new('notes-update', 'notes-update', [CompletionResultType]::ParameterValue, 'Update a note')
            [CompletionResult]::new('notes-delete', 'notes-delete', [CompletionResultType]::ParameterValue, 'Delete a note')
            [CompletionResult]::new('manuals-add', 'manuals-add', [CompletionResultType]::ParameterValue, 'Upload a manual file (`POST /api/roms/{id}/manuals`)')
            [CompletionResult]::new('cover-search', 'cover-search', [CompletionResultType]::ParameterValue, 'Search covers and metadata matches')
            [CompletionResult]::new('upload', 'upload', [CompletionResultType]::ParameterValue, 'Upload a ROM file to a platform')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;r;help;get' {
            break
        }
        'romm-cli;r;help;find' {
            break
        }
        'romm-cli;r;help;filters' {
            break
        }
        'romm-cli;r;help;delete' {
            break
        }
        'romm-cli;r;help;props' {
            break
        }
        'romm-cli;r;help;notes-list' {
            break
        }
        'romm-cli;r;help;notes-add' {
            break
        }
        'romm-cli;r;help;notes-update' {
            break
        }
        'romm-cli;r;help;notes-delete' {
            break
        }
        'romm-cli;r;help;manuals-add' {
            break
        }
        'romm-cli;r;help;cover-search' {
            break
        }
        'romm-cli;r;help;upload' {
            break
        }
        'romm-cli;r;help;help' {
            break
        }
        'romm-cli;scan' {
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Restrict scan to one or more platform slugs (comma-separated); passed as `platform_slugs` task kwargs')
            [CompletionResult]::new('--wait-timeout-secs', '--wait-timeout-secs', [CompletionResultType]::ParameterName, 'Max seconds to wait when `--wait` is set (default: 3600)')
            [CompletionResult]::new('--wait', '--wait', [CompletionResultType]::ParameterName, 'Wait until the scan task completes (polls every 2 seconds)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;sync' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('device', 'device', [CompletionResultType]::ParameterValue, 'Manage sync devices')
            [CompletionResult]::new('plan', 'plan', [CompletionResultType]::ParameterValue, 'Negotiate sync operations without modifying files')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Execute one-shot sync operations')
            [CompletionResult]::new('sessions', 'sessions', [CompletionResultType]::ParameterValue, 'Inspect sync sessions')
            [CompletionResult]::new('push-pull', 'push-pull', [CompletionResultType]::ParameterValue, 'Trigger push-pull mode on a registered device')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;sync;device' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('register', 'register', [CompletionResultType]::ParameterValue, 'Register a device (`POST /api/devices`)')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List devices (`GET /api/devices`)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get one device (`GET /api/devices/{id}`)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;sync;device;register' {
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'name')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'platform')
            [CompletionResult]::new('--client', '--client', [CompletionResultType]::ParameterName, 'client')
            [CompletionResult]::new('--client-version', '--client-version', [CompletionResultType]::ParameterName, 'client-version')
            [CompletionResult]::new('--hostname', '--hostname', [CompletionResultType]::ParameterName, 'hostname')
            [CompletionResult]::new('--mac-address', '--mac-address', [CompletionResultType]::ParameterName, 'mac-address')
            [CompletionResult]::new('--ip-address', '--ip-address', [CompletionResultType]::ParameterName, 'ip-address')
            [CompletionResult]::new('--sync-mode', '--sync-mode', [CompletionResultType]::ParameterName, 'sync-mode')
            [CompletionResult]::new('--sync-config-json', '--sync-config-json', [CompletionResultType]::ParameterName, 'Optional JSON object string for `sync_config`')
            [CompletionResult]::new('--allow-duplicate', '--allow-duplicate', [CompletionResultType]::ParameterName, 'allow-duplicate')
            [CompletionResult]::new('--reset-syncs', '--reset-syncs', [CompletionResultType]::ParameterName, 'reset-syncs')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;sync;device;list' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;sync;device;get' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;sync;device;help' {
            [CompletionResult]::new('register', 'register', [CompletionResultType]::ParameterValue, 'Register a device (`POST /api/devices`)')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List devices (`GET /api/devices`)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get one device (`GET /api/devices/{id}`)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;sync;device;help;register' {
            break
        }
        'romm-cli;sync;device;help;list' {
            break
        }
        'romm-cli;sync;device;help;get' {
            break
        }
        'romm-cli;sync;device;help;help' {
            break
        }
        'romm-cli;sync;plan' {
            [CompletionResult]::new('--device-id', '--device-id', [CompletionResultType]::ParameterName, 'device-id')
            [CompletionResult]::new('--manifest', '--manifest', [CompletionResultType]::ParameterName, 'manifest')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;sync;run' {
            [CompletionResult]::new('--device-id', '--device-id', [CompletionResultType]::ParameterName, 'device-id')
            [CompletionResult]::new('--manifest', '--manifest', [CompletionResultType]::ParameterName, 'manifest')
            [CompletionResult]::new('--download-dir', '--download-dir', [CompletionResultType]::ParameterName, 'Folder for downloaded saves (defaults to the manifest directory)')
            [CompletionResult]::new('--conflict', '--conflict', [CompletionResultType]::ParameterName, 'conflict')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;sync;sessions' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List sessions (`GET /api/sync/sessions`)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get one session (`GET /api/sync/sessions/{id}`)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;sync;sessions;list' {
            [CompletionResult]::new('--device-id', '--device-id', [CompletionResultType]::ParameterName, 'device-id')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'limit')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;sync;sessions;get' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;sync;sessions;help' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List sessions (`GET /api/sync/sessions`)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get one session (`GET /api/sync/sessions/{id}`)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;sync;sessions;help;list' {
            break
        }
        'romm-cli;sync;sessions;help;get' {
            break
        }
        'romm-cli;sync;sessions;help;help' {
            break
        }
        'romm-cli;sync;push-pull' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON (overrides global --json when set)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;sync;help' {
            [CompletionResult]::new('device', 'device', [CompletionResultType]::ParameterValue, 'Manage sync devices')
            [CompletionResult]::new('plan', 'plan', [CompletionResultType]::ParameterValue, 'Negotiate sync operations without modifying files')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Execute one-shot sync operations')
            [CompletionResult]::new('sessions', 'sessions', [CompletionResultType]::ParameterValue, 'Inspect sync sessions')
            [CompletionResult]::new('push-pull', 'push-pull', [CompletionResultType]::ParameterValue, 'Trigger push-pull mode on a registered device')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;sync;help;device' {
            [CompletionResult]::new('register', 'register', [CompletionResultType]::ParameterValue, 'Register a device (`POST /api/devices`)')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List devices (`GET /api/devices`)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get one device (`GET /api/devices/{id}`)')
            break
        }
        'romm-cli;sync;help;device;register' {
            break
        }
        'romm-cli;sync;help;device;list' {
            break
        }
        'romm-cli;sync;help;device;get' {
            break
        }
        'romm-cli;sync;help;plan' {
            break
        }
        'romm-cli;sync;help;run' {
            break
        }
        'romm-cli;sync;help;sessions' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List sessions (`GET /api/sync/sessions`)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get one session (`GET /api/sync/sessions/{id}`)')
            break
        }
        'romm-cli;sync;help;sessions;list' {
            break
        }
        'romm-cli;sync;help;sessions;get' {
            break
        }
        'romm-cli;sync;help;push-pull' {
            break
        }
        'romm-cli;sync;help;help' {
            break
        }
        'romm-cli;download' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('batch', 'batch', [CompletionResultType]::ParameterValue, 'Download multiple ROMs matching filters')
            [CompletionResult]::new('all', 'all', [CompletionResultType]::ParameterValue, 'Download multiple ROMs matching filters')
            [CompletionResult]::new('extras', 'extras', [CompletionResultType]::ParameterValue, 'Download covers, manuals, updates, and DLC for one game')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;dl' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('batch', 'batch', [CompletionResultType]::ParameterValue, 'Download multiple ROMs matching filters')
            [CompletionResult]::new('all', 'all', [CompletionResultType]::ParameterValue, 'Download multiple ROMs matching filters')
            [CompletionResult]::new('extras', 'extras', [CompletionResultType]::ParameterValue, 'Download covers, manuals, updates, and DLC for one game')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;get' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('batch', 'batch', [CompletionResultType]::ParameterValue, 'Download multiple ROMs matching filters')
            [CompletionResult]::new('all', 'all', [CompletionResultType]::ParameterValue, 'Download multiple ROMs matching filters')
            [CompletionResult]::new('extras', 'extras', [CompletionResultType]::ParameterValue, 'Download covers, manuals, updates, and DLC for one game')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;download;batch' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'romm-cli;download;all' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'romm-cli;dl;batch' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'romm-cli;dl;all' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'romm-cli;get;batch' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'romm-cli;get;all' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'romm-cli;download;extras' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'romm-cli;dl;extras' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'romm-cli;get;extras' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Directory to save the ROM zip(s) to')
            [CompletionResult]::new('--platform', '--platform', [CompletionResultType]::ParameterName, 'Filter by platform slug or title (e.g. "3ds")')
            [CompletionResult]::new('--search-term', '--search-term', [CompletionResultType]::ParameterName, 'Filter by search term')
            [CompletionResult]::new('--jobs', '--jobs', [CompletionResultType]::ParameterName, 'Maximum concurrent downloads (default: 4)')
            [CompletionResult]::new('--extract-layout', '--extract-layout', [CompletionResultType]::ParameterName, 'Layout for extracted files when --extract is set (default: platform)')
            [CompletionResult]::new('--extract', '--extract', [CompletionResultType]::ParameterName, 'Extract each downloaded ZIP after download completes (batch mode only)')
            [CompletionResult]::new('--delete-zip-after-extract', '--delete-zip-after-extract', [CompletionResultType]::ParameterName, 'Delete ZIP files after successful extraction (batch mode only)')
            [CompletionResult]::new('--with-extras', '--with-extras', [CompletionResultType]::ParameterName, 'Include updates and DLC after downloading the base game (single-ROM mode)')
            [CompletionResult]::new('--no-extras', '--no-extras', [CompletionResultType]::ParameterName, 'Skip updates and DLC (single-ROM mode)')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Assume yes for extras prompt (single-ROM mode)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'romm-cli;download;help' {
            [CompletionResult]::new('batch', 'batch', [CompletionResultType]::ParameterValue, 'Download multiple ROMs matching filters')
            [CompletionResult]::new('extras', 'extras', [CompletionResultType]::ParameterValue, 'Download covers, manuals, updates, and DLC for one game')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;download;help;batch' {
            break
        }
        'romm-cli;download;help;extras' {
            break
        }
        'romm-cli;download;help;help' {
            break
        }
        'romm-cli;dl;help' {
            [CompletionResult]::new('batch', 'batch', [CompletionResultType]::ParameterValue, 'Download multiple ROMs matching filters')
            [CompletionResult]::new('extras', 'extras', [CompletionResultType]::ParameterValue, 'Download covers, manuals, updates, and DLC for one game')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;dl;help;batch' {
            break
        }
        'romm-cli;dl;help;extras' {
            break
        }
        'romm-cli;dl;help;help' {
            break
        }
        'romm-cli;get;help' {
            [CompletionResult]::new('batch', 'batch', [CompletionResultType]::ParameterValue, 'Download multiple ROMs matching filters')
            [CompletionResult]::new('extras', 'extras', [CompletionResultType]::ParameterValue, 'Download covers, manuals, updates, and DLC for one game')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;get;help;batch' {
            break
        }
        'romm-cli;get;help;extras' {
            break
        }
        'romm-cli;get;help;help' {
            break
        }
        'romm-cli;cache' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Print the effective ROM cache file path')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Show ROM cache metadata and parse status')
            [CompletionResult]::new('clear', 'clear', [CompletionResultType]::ParameterValue, 'Delete the ROM cache file if it exists')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;cache;path' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;cache;info' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;cache;clear' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;cache;help' {
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Print the effective ROM cache file path')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Show ROM cache metadata and parse status')
            [CompletionResult]::new('clear', 'clear', [CompletionResultType]::ParameterValue, 'Delete the ROM cache file if it exists')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;cache;help;path' {
            break
        }
        'romm-cli;cache;help;info' {
            break
        }
        'romm-cli;cache;help;clear' {
            break
        }
        'romm-cli;cache;help;help' {
            break
        }
        'romm-cli;auth' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('login', 'login', [CompletionResultType]::ParameterValue, 'Set/rotate authentication credentials (Bearer, Basic, API key, or pairing code)')
            [CompletionResult]::new('logout', 'logout', [CompletionResultType]::ParameterValue, 'Remove stored authentication (leaves non-auth config untouched)')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current authentication mode and where it comes from (env/config/keyring)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;auth;login' {
            [CompletionResult]::new('--token', '--token', [CompletionResultType]::ParameterName, 'API token (Bearer). Skips interactive prompts')
            [CompletionResult]::new('--token-file', '--token-file', [CompletionResultType]::ParameterName, 'Read API token (Bearer) from UTF-8 file. Use ''-'' for stdin')
            [CompletionResult]::new('--username', '--username', [CompletionResultType]::ParameterName, 'Basic auth username')
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'Basic auth password (discouraged: visible in process list)')
            [CompletionResult]::new('--password-file', '--password-file', [CompletionResultType]::ParameterName, 'Read Basic auth password from a UTF-8 file. Use ''-'' for stdin')
            [CompletionResult]::new('--api-key-header', '--api-key-header', [CompletionResultType]::ParameterName, 'API key header name (e.g. X-API-Key)')
            [CompletionResult]::new('--api-key', '--api-key', [CompletionResultType]::ParameterName, 'API key value')
            [CompletionResult]::new('--pairing-code', '--pairing-code', [CompletionResultType]::ParameterName, 'Web UI pairing code (8 characters)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;auth;logout' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;auth;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;auth;help' {
            [CompletionResult]::new('login', 'login', [CompletionResultType]::ParameterValue, 'Set/rotate authentication credentials (Bearer, Basic, API key, or pairing code)')
            [CompletionResult]::new('logout', 'logout', [CompletionResultType]::ParameterValue, 'Remove stored authentication (leaves non-auth config untouched)')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current authentication mode and where it comes from (env/config/keyring)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;auth;help;login' {
            break
        }
        'romm-cli;auth;help;logout' {
            break
        }
        'romm-cli;auth;help;status' {
            break
        }
        'romm-cli;auth;help;help' {
            break
        }
        'romm-cli;update' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;completions' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase output verbosity (logs requests to stderr)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output JSON instead of human-readable text where supported')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'romm-cli;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Create or update user configuration')
            [CompletionResult]::new('api', 'api', [CompletionResultType]::ParameterValue, 'Low-level access to any RomM API endpoint')
            [CompletionResult]::new('platforms', 'platforms', [CompletionResultType]::ParameterValue, 'Manage gaming platforms')
            [CompletionResult]::new('roms', 'roms', [CompletionResultType]::ParameterValue, 'Manage ROM files and metadata')
            [CompletionResult]::new('scan', 'scan', [CompletionResultType]::ParameterValue, 'Trigger a library scan on the RomM server')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Save-sync workflows (device registration, planning, and execution)')
            [CompletionResult]::new('download', 'download', [CompletionResultType]::ParameterValue, 'Download a ROM or related extras from the server')
            [CompletionResult]::new('cache', 'cache', [CompletionResultType]::ParameterValue, 'Manage the local persistent cache')
            [CompletionResult]::new('auth', 'auth', [CompletionResultType]::ParameterValue, 'Manage authentication credentials')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Check for and install application updates')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'romm-cli;help;init' {
            break
        }
        'romm-cli;help;api' {
            [CompletionResult]::new('call', 'call', [CompletionResultType]::ParameterValue, 'Make a generic API call')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Shortcut for GET request')
            [CompletionResult]::new('post', 'post', [CompletionResultType]::ParameterValue, 'Shortcut for POST request')
            break
        }
        'romm-cli;help;api;call' {
            break
        }
        'romm-cli;help;api;get' {
            break
        }
        'romm-cli;help;api;post' {
            break
        }
        'romm-cli;help;platforms' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all platforms (default)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get details for a specific platform')
            break
        }
        'romm-cli;help;platforms;list' {
            break
        }
        'romm-cli;help;platforms;get' {
            break
        }
        'romm-cli;help;roms' {
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get detailed information for a single ROM')
            [CompletionResult]::new('find', 'find', [CompletionResultType]::ParameterValue, 'Lookup ROM by file hash or metadata provider id')
            [CompletionResult]::new('filters', 'filters', [CompletionResultType]::ParameterValue, 'Print canonical filter values from `GET /api/roms/filters`')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete ROMs from the database (optional filesystem delete)')
            [CompletionResult]::new('props', 'props', [CompletionResultType]::ParameterValue, 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)')
            [CompletionResult]::new('notes-list', 'notes-list', [CompletionResultType]::ParameterValue, 'List notes for a ROM')
            [CompletionResult]::new('notes-add', 'notes-add', [CompletionResultType]::ParameterValue, 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})')
            [CompletionResult]::new('notes-update', 'notes-update', [CompletionResultType]::ParameterValue, 'Update a note')
            [CompletionResult]::new('notes-delete', 'notes-delete', [CompletionResultType]::ParameterValue, 'Delete a note')
            [CompletionResult]::new('manuals-add', 'manuals-add', [CompletionResultType]::ParameterValue, 'Upload a manual file (`POST /api/roms/{id}/manuals`)')
            [CompletionResult]::new('cover-search', 'cover-search', [CompletionResultType]::ParameterValue, 'Search covers and metadata matches')
            [CompletionResult]::new('upload', 'upload', [CompletionResultType]::ParameterValue, 'Upload a ROM file to a platform')
            break
        }
        'romm-cli;help;roms;get' {
            break
        }
        'romm-cli;help;roms;find' {
            break
        }
        'romm-cli;help;roms;filters' {
            break
        }
        'romm-cli;help;roms;delete' {
            break
        }
        'romm-cli;help;roms;props' {
            break
        }
        'romm-cli;help;roms;notes-list' {
            break
        }
        'romm-cli;help;roms;notes-add' {
            break
        }
        'romm-cli;help;roms;notes-update' {
            break
        }
        'romm-cli;help;roms;notes-delete' {
            break
        }
        'romm-cli;help;roms;manuals-add' {
            break
        }
        'romm-cli;help;roms;cover-search' {
            break
        }
        'romm-cli;help;roms;upload' {
            break
        }
        'romm-cli;help;scan' {
            break
        }
        'romm-cli;help;sync' {
            [CompletionResult]::new('device', 'device', [CompletionResultType]::ParameterValue, 'Manage sync devices')
            [CompletionResult]::new('plan', 'plan', [CompletionResultType]::ParameterValue, 'Negotiate sync operations without modifying files')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, 'Execute one-shot sync operations')
            [CompletionResult]::new('sessions', 'sessions', [CompletionResultType]::ParameterValue, 'Inspect sync sessions')
            [CompletionResult]::new('push-pull', 'push-pull', [CompletionResultType]::ParameterValue, 'Trigger push-pull mode on a registered device')
            break
        }
        'romm-cli;help;sync;device' {
            [CompletionResult]::new('register', 'register', [CompletionResultType]::ParameterValue, 'Register a device (`POST /api/devices`)')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List devices (`GET /api/devices`)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get one device (`GET /api/devices/{id}`)')
            break
        }
        'romm-cli;help;sync;device;register' {
            break
        }
        'romm-cli;help;sync;device;list' {
            break
        }
        'romm-cli;help;sync;device;get' {
            break
        }
        'romm-cli;help;sync;plan' {
            break
        }
        'romm-cli;help;sync;run' {
            break
        }
        'romm-cli;help;sync;sessions' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List sessions (`GET /api/sync/sessions`)')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Get one session (`GET /api/sync/sessions/{id}`)')
            break
        }
        'romm-cli;help;sync;sessions;list' {
            break
        }
        'romm-cli;help;sync;sessions;get' {
            break
        }
        'romm-cli;help;sync;push-pull' {
            break
        }
        'romm-cli;help;download' {
            [CompletionResult]::new('batch', 'batch', [CompletionResultType]::ParameterValue, 'Download multiple ROMs matching filters')
            [CompletionResult]::new('extras', 'extras', [CompletionResultType]::ParameterValue, 'Download covers, manuals, updates, and DLC for one game')
            break
        }
        'romm-cli;help;download;batch' {
            break
        }
        'romm-cli;help;download;extras' {
            break
        }
        'romm-cli;help;cache' {
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Print the effective ROM cache file path')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'Show ROM cache metadata and parse status')
            [CompletionResult]::new('clear', 'clear', [CompletionResultType]::ParameterValue, 'Delete the ROM cache file if it exists')
            break
        }
        'romm-cli;help;cache;path' {
            break
        }
        'romm-cli;help;cache;info' {
            break
        }
        'romm-cli;help;cache;clear' {
            break
        }
        'romm-cli;help;auth' {
            [CompletionResult]::new('login', 'login', [CompletionResultType]::ParameterValue, 'Set/rotate authentication credentials (Bearer, Basic, API key, or pairing code)')
            [CompletionResult]::new('logout', 'logout', [CompletionResultType]::ParameterValue, 'Remove stored authentication (leaves non-auth config untouched)')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current authentication mode and where it comes from (env/config/keyring)')
            break
        }
        'romm-cli;help;auth;login' {
            break
        }
        'romm-cli;help;auth;logout' {
            break
        }
        'romm-cli;help;auth;status' {
            break
        }
        'romm-cli;help;update' {
            break
        }
        'romm-cli;help;completions' {
            break
        }
        'romm-cli;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
