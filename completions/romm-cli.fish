# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_romm_cli_global_optspecs
	string join \n v/verbose json h/help V/version
end

function __fish_romm_cli_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_romm_cli_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_romm_cli_using_subcommand
	set -l cmd (__fish_romm_cli_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c romm-cli -n "__fish_romm_cli_needs_command" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -s V -l version -d 'Print version'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "init" -d 'Create or update user configuration'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "setup" -d 'Create or update user configuration'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "tui" -d 'Launch the interactive Terminal User Interface (TUI)'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "api" -d 'Low-level access to any RomM API endpoint'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "call" -d 'Low-level access to any RomM API endpoint'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "platforms" -d 'Manage gaming platforms'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "platform" -d 'Manage gaming platforms'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "p" -d 'Manage gaming platforms'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "plats" -d 'Manage gaming platforms'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "roms" -d 'Manage ROM files and metadata'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "rom" -d 'Manage ROM files and metadata'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "r" -d 'Manage ROM files and metadata'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "scan" -d 'Trigger a library scan on the RomM server'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "sync" -d 'Save-sync workflows (device registration, planning, and execution)'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "download" -d 'Download a ROM or related extras from the server'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "dl" -d 'Download a ROM or related extras from the server'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "get" -d 'Download a ROM or related extras from the server'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "cache" -d 'Manage the local persistent cache'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "auth" -d 'Manage authentication credentials'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "update" -d 'Check for and install application updates'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "completions" -d 'Generate shell completion scripts'
complete -c romm-cli -n "__fish_romm_cli_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -l url -d 'RomM origin URL (e.g. <https://romm.example>). If provided with a token, skips interactive prompts' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -l token -d 'Bearer token string (discouraged: visible in process list)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -l token-file -d 'Read Bearer token from a UTF-8 file. Use \'-\' for stdin' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -l download-dir -d 'ROMs directory' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -l force -d 'Overwrite existing user config `config.json` without asking'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -l print-path -d 'Print the path to the user config `config.json` and exit'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -l no-https -d 'Disable HTTPS (use HTTP instead)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -l check -d 'Verify URL and token by fetching OpenAPI after saving'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand init" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -l url -d 'RomM origin URL (e.g. <https://romm.example>). If provided with a token, skips interactive prompts' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -l token -d 'Bearer token string (discouraged: visible in process list)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -l token-file -d 'Read Bearer token from a UTF-8 file. Use \'-\' for stdin' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -l download-dir -d 'ROMs directory' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -l force -d 'Overwrite existing user config `config.json` without asking'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -l print-path -d 'Print the path to the user config `config.json` and exit'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -l no-https -d 'Disable HTTPS (use HTTP instead)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -l check -d 'Verify URL and token by fetching OpenAPI after saving'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand setup" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand tui" -l mock-update -d 'Force show a fake update prompt for UI testing'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand tui" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand tui" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand tui" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and not __fish_seen_subcommand_from call get post help" -l query -d 'Query parameters as key=value, repeatable' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and not __fish_seen_subcommand_from call get post help" -l data -d 'JSON request body as a string' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and not __fish_seen_subcommand_from call get post help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and not __fish_seen_subcommand_from call get post help" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and not __fish_seen_subcommand_from call get post help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and not __fish_seen_subcommand_from call get post help" -a "call" -d 'Make a generic API call'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and not __fish_seen_subcommand_from call get post help" -a "get" -d 'Shortcut for GET request'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and not __fish_seen_subcommand_from call get post help" -a "post" -d 'Shortcut for POST request'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and not __fish_seen_subcommand_from call get post help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from call" -l query -d 'Query parameters as key=value, repeatable' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from call" -l data -d 'JSON request body as a string' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from call" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from call" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from call" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from get" -l query -d 'Query parameters as key=value, repeatable' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from get" -l data -d 'JSON request body as a string' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from get" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from post" -l query -d 'Query parameters as key=value, repeatable' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from post" -l data -d 'JSON request body as a string' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from post" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from post" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from post" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from help" -f -a "call" -d 'Make a generic API call'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from help" -f -a "get" -d 'Shortcut for GET request'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from help" -f -a "post" -d 'Shortcut for POST request'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand api; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and not __fish_seen_subcommand_from call get post help" -l query -d 'Query parameters as key=value, repeatable' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and not __fish_seen_subcommand_from call get post help" -l data -d 'JSON request body as a string' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and not __fish_seen_subcommand_from call get post help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and not __fish_seen_subcommand_from call get post help" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and not __fish_seen_subcommand_from call get post help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and not __fish_seen_subcommand_from call get post help" -a "call" -d 'Make a generic API call'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and not __fish_seen_subcommand_from call get post help" -a "get" -d 'Shortcut for GET request'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and not __fish_seen_subcommand_from call get post help" -a "post" -d 'Shortcut for POST request'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and not __fish_seen_subcommand_from call get post help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from call" -l query -d 'Query parameters as key=value, repeatable' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from call" -l data -d 'JSON request body as a string' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from call" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from call" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from call" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from get" -l query -d 'Query parameters as key=value, repeatable' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from get" -l data -d 'JSON request body as a string' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from get" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from post" -l query -d 'Query parameters as key=value, repeatable' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from post" -l data -d 'JSON request body as a string' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from post" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from post" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from post" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from help" -f -a "call" -d 'Make a generic API call'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from help" -f -a "get" -d 'Shortcut for GET request'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from help" -f -a "post" -d 'Shortcut for POST request'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand call; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and not __fish_seen_subcommand_from list ls get info help" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and not __fish_seen_subcommand_from list ls get info help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and not __fish_seen_subcommand_from list ls get info help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and not __fish_seen_subcommand_from list ls get info help" -f -a "list" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and not __fish_seen_subcommand_from list ls get info help" -f -a "ls" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and not __fish_seen_subcommand_from list ls get info help" -f -a "get" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and not __fish_seen_subcommand_from list ls get info help" -f -a "info" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and not __fish_seen_subcommand_from list ls get info help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from ls" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from ls" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from get" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from info" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platforms; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and not __fish_seen_subcommand_from list ls get info help" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and not __fish_seen_subcommand_from list ls get info help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and not __fish_seen_subcommand_from list ls get info help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and not __fish_seen_subcommand_from list ls get info help" -f -a "list" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and not __fish_seen_subcommand_from list ls get info help" -f -a "ls" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and not __fish_seen_subcommand_from list ls get info help" -f -a "get" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and not __fish_seen_subcommand_from list ls get info help" -f -a "info" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and not __fish_seen_subcommand_from list ls get info help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from ls" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from ls" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from get" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from info" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand platform; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and not __fish_seen_subcommand_from list ls get info help" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and not __fish_seen_subcommand_from list ls get info help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and not __fish_seen_subcommand_from list ls get info help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and not __fish_seen_subcommand_from list ls get info help" -f -a "list" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and not __fish_seen_subcommand_from list ls get info help" -f -a "ls" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and not __fish_seen_subcommand_from list ls get info help" -f -a "get" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and not __fish_seen_subcommand_from list ls get info help" -f -a "info" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and not __fish_seen_subcommand_from list ls get info help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from ls" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from ls" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from get" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from info" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and not __fish_seen_subcommand_from list ls get info help" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and not __fish_seen_subcommand_from list ls get info help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and not __fish_seen_subcommand_from list ls get info help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and not __fish_seen_subcommand_from list ls get info help" -f -a "list" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and not __fish_seen_subcommand_from list ls get info help" -f -a "ls" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and not __fish_seen_subcommand_from list ls get info help" -f -a "get" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and not __fish_seen_subcommand_from list ls get info help" -f -a "info" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and not __fish_seen_subcommand_from list ls get info help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from ls" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from ls" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from get" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from info" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from help" -f -a "list" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand plats; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l search-term -l query -l q -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l platform -l p -d 'Platform slug or name; repeat for multiple `platform_ids`' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l collection -d 'Manual collection id or exact name' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l smart-collection -d 'Smart collection id or exact name' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l virtual-collection -d 'Virtual collection id (e.g. recent)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l limit -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l offset -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l matched -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l favorite -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l duplicate -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l last-played -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l playable -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l missing -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l has-ra -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l verified -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l group-by-meta-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l with-char-index -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l with-filter-values -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l genre -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l franchise -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l collection-tag -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l company -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l age-rating -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l status -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l region -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l language -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l player-count -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l genres-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l franchises-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l collections-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l companies-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l age-ratings-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l regions-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l languages-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l statuses-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l player-counts-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l order-by -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l order-dir -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l updated-after -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "get" -d 'Get detailed information for a single ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "info" -d 'Get detailed information for a single ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "find" -d 'Lookup ROM by file hash or metadata provider id'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "filters" -d 'Print canonical filter values from `GET /api/roms/filters`'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "delete" -d 'Delete ROMs from the database (optional filesystem delete)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "props" -d 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-list" -d 'List notes for a ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-add" -d 'Add a note (JSON body string, e.g. {\\"title\\":\\"t\\",\\"content\\":\\"c\\"})'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-update" -d 'Update a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-delete" -d 'Delete a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "manuals-add" -d 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "cover-search" -d 'Search covers and metadata matches'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "upload" -d 'Upload a ROM file to a platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "up" -d 'Upload a ROM file to a platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from get" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from info" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l crc -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l md5 -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l sha1 -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l igdb-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l moby-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l ss-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l ra-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l launchbox-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l hasheous-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l tgdb-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l flashpoint-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l hltb-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from find" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from filters" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from filters" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from filters" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from delete" -l delete-from-fs -d 'Also delete these ROM ids from disk (repeat ids as needed)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from delete" -l yes -d 'Skip confirmation'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from delete" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from delete" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l is-main-sibling -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l backlogged -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l now-playing -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l hidden -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l rating -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l difficulty -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l completion -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l status -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l update-last-played
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l remove-last-played
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from props" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-list" -l public-only -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-list" -l search -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-list" -l tag -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-list" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-list" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-list" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-add" -l json -d 'JSON object' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-add" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-add" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-update" -l json -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-update" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-update" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-delete" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-delete" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from notes-delete" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from manuals-add" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from manuals-add" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from manuals-add" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from cover-search" -l query -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from cover-search" -l search-by -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from cover-search" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from cover-search" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from cover-search" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from upload" -l platform -d 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from upload" -l wait-timeout-secs -d 'Max seconds to wait when `--wait` is set (default: 3600)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from upload" -s s -l scan -d 'Trigger a library scan after upload completes'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from upload" -l wait -d 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from upload" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from upload" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from upload" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from up" -l platform -d 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from up" -l wait-timeout-secs -d 'Max seconds to wait when `--wait` is set (default: 3600)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from up" -s s -l scan -d 'Trigger a library scan after upload completes'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from up" -l wait -d 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from up" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from up" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from up" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get detailed information for a single ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "find" -d 'Lookup ROM by file hash or metadata provider id'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "filters" -d 'Print canonical filter values from `GET /api/roms/filters`'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "delete" -d 'Delete ROMs from the database (optional filesystem delete)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "props" -d 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "notes-list" -d 'List notes for a ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "notes-add" -d 'Add a note (JSON body string, e.g. {\\"title\\":\\"t\\",\\"content\\":\\"c\\"})'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "notes-update" -d 'Update a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "notes-delete" -d 'Delete a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "manuals-add" -d 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "cover-search" -d 'Search covers and metadata matches'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "upload" -d 'Upload a ROM file to a platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand roms; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l search-term -l query -l q -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l platform -l p -d 'Platform slug or name; repeat for multiple `platform_ids`' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l collection -d 'Manual collection id or exact name' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l smart-collection -d 'Smart collection id or exact name' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l virtual-collection -d 'Virtual collection id (e.g. recent)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l limit -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l offset -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l matched -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l favorite -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l duplicate -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l last-played -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l playable -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l missing -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l has-ra -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l verified -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l group-by-meta-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l with-char-index -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l with-filter-values -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l genre -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l franchise -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l collection-tag -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l company -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l age-rating -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l status -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l region -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l language -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l player-count -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l genres-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l franchises-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l collections-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l companies-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l age-ratings-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l regions-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l languages-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l statuses-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l player-counts-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l order-by -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l order-dir -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l updated-after -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "get" -d 'Get detailed information for a single ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "info" -d 'Get detailed information for a single ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "find" -d 'Lookup ROM by file hash or metadata provider id'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "filters" -d 'Print canonical filter values from `GET /api/roms/filters`'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "delete" -d 'Delete ROMs from the database (optional filesystem delete)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "props" -d 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-list" -d 'List notes for a ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-add" -d 'Add a note (JSON body string, e.g. {\\"title\\":\\"t\\",\\"content\\":\\"c\\"})'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-update" -d 'Update a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-delete" -d 'Delete a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "manuals-add" -d 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "cover-search" -d 'Search covers and metadata matches'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "upload" -d 'Upload a ROM file to a platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "up" -d 'Upload a ROM file to a platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from get" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from info" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l crc -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l md5 -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l sha1 -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l igdb-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l moby-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l ss-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l ra-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l launchbox-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l hasheous-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l tgdb-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l flashpoint-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l hltb-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from find" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from filters" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from filters" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from filters" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from delete" -l delete-from-fs -d 'Also delete these ROM ids from disk (repeat ids as needed)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from delete" -l yes -d 'Skip confirmation'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from delete" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from delete" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l is-main-sibling -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l backlogged -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l now-playing -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l hidden -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l rating -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l difficulty -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l completion -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l status -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l update-last-played
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l remove-last-played
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from props" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-list" -l public-only -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-list" -l search -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-list" -l tag -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-list" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-list" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-list" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-add" -l json -d 'JSON object' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-add" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-add" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-update" -l json -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-update" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-update" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-delete" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-delete" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from notes-delete" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from manuals-add" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from manuals-add" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from manuals-add" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from cover-search" -l query -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from cover-search" -l search-by -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from cover-search" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from cover-search" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from cover-search" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from upload" -l platform -d 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from upload" -l wait-timeout-secs -d 'Max seconds to wait when `--wait` is set (default: 3600)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from upload" -s s -l scan -d 'Trigger a library scan after upload completes'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from upload" -l wait -d 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from upload" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from upload" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from upload" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from up" -l platform -d 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from up" -l wait-timeout-secs -d 'Max seconds to wait when `--wait` is set (default: 3600)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from up" -s s -l scan -d 'Trigger a library scan after upload completes'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from up" -l wait -d 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from up" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from up" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from up" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get detailed information for a single ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "find" -d 'Lookup ROM by file hash or metadata provider id'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "filters" -d 'Print canonical filter values from `GET /api/roms/filters`'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "delete" -d 'Delete ROMs from the database (optional filesystem delete)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "props" -d 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "notes-list" -d 'List notes for a ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "notes-add" -d 'Add a note (JSON body string, e.g. {\\"title\\":\\"t\\",\\"content\\":\\"c\\"})'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "notes-update" -d 'Update a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "notes-delete" -d 'Delete a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "manuals-add" -d 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "cover-search" -d 'Search covers and metadata matches'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "upload" -d 'Upload a ROM file to a platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand rom; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l search-term -l query -l q -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l platform -l p -d 'Platform slug or name; repeat for multiple `platform_ids`' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l collection -d 'Manual collection id or exact name' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l smart-collection -d 'Smart collection id or exact name' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l virtual-collection -d 'Virtual collection id (e.g. recent)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l limit -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l offset -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l matched -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l favorite -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l duplicate -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l last-played -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l playable -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l missing -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l has-ra -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l verified -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l group-by-meta-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l with-char-index -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l with-filter-values -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l genre -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l franchise -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l collection-tag -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l company -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l age-rating -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l status -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l region -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l language -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l player-count -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l genres-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l franchises-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l collections-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l companies-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l age-ratings-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l regions-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l languages-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l statuses-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l player-counts-logic -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l order-by -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l order-dir -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l updated-after -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "get" -d 'Get detailed information for a single ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "info" -d 'Get detailed information for a single ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "find" -d 'Lookup ROM by file hash or metadata provider id'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "filters" -d 'Print canonical filter values from `GET /api/roms/filters`'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "delete" -d 'Delete ROMs from the database (optional filesystem delete)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "props" -d 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-list" -d 'List notes for a ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-add" -d 'Add a note (JSON body string, e.g. {\\"title\\":\\"t\\",\\"content\\":\\"c\\"})'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-update" -d 'Update a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "notes-delete" -d 'Delete a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "manuals-add" -d 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "cover-search" -d 'Search covers and metadata matches'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "upload" -d 'Upload a ROM file to a platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "up" -d 'Upload a ROM file to a platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and not __fish_seen_subcommand_from get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from get" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from info" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l crc -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l md5 -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l sha1 -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l igdb-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l moby-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l ss-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l ra-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l launchbox-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l hasheous-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l tgdb-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l flashpoint-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l hltb-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from find" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from filters" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from filters" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from filters" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from delete" -l delete-from-fs -d 'Also delete these ROM ids from disk (repeat ids as needed)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from delete" -l yes -d 'Skip confirmation'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from delete" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from delete" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l is-main-sibling -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l backlogged -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l now-playing -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l hidden -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l rating -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l difficulty -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l completion -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l status -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l update-last-played
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l remove-last-played
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from props" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-list" -l public-only -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-list" -l search -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-list" -l tag -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-list" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-list" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-list" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-add" -l json -d 'JSON object' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-add" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-add" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-update" -l json -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-update" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-update" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-delete" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-delete" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from notes-delete" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from manuals-add" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from manuals-add" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from manuals-add" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from cover-search" -l query -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from cover-search" -l search-by -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from cover-search" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from cover-search" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from cover-search" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from upload" -l platform -d 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from upload" -l wait-timeout-secs -d 'Max seconds to wait when `--wait` is set (default: 3600)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from upload" -s s -l scan -d 'Trigger a library scan after upload completes'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from upload" -l wait -d 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from upload" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from upload" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from upload" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from up" -l platform -d 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from up" -l wait-timeout-secs -d 'Max seconds to wait when `--wait` is set (default: 3600)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from up" -s s -l scan -d 'Trigger a library scan after upload completes'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from up" -l wait -d 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from up" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from up" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from up" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "get" -d 'Get detailed information for a single ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "find" -d 'Lookup ROM by file hash or metadata provider id'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "filters" -d 'Print canonical filter values from `GET /api/roms/filters`'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "delete" -d 'Delete ROMs from the database (optional filesystem delete)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "props" -d 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "notes-list" -d 'List notes for a ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "notes-add" -d 'Add a note (JSON body string, e.g. {\\"title\\":\\"t\\",\\"content\\":\\"c\\"})'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "notes-update" -d 'Update a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "notes-delete" -d 'Delete a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "manuals-add" -d 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "cover-search" -d 'Search covers and metadata matches'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "upload" -d 'Upload a ROM file to a platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand scan" -l platform -d 'Restrict scan to one or more platform slugs (comma-separated); passed as `platform_slugs` task kwargs' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand scan" -l wait-timeout-secs -d 'Max seconds to wait when `--wait` is set (default: 3600)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand scan" -l wait -d 'Wait until the scan task completes (polls every 2 seconds)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand scan" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand scan" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand scan" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and not __fish_seen_subcommand_from device plan run sessions push-pull help" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and not __fish_seen_subcommand_from device plan run sessions push-pull help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and not __fish_seen_subcommand_from device plan run sessions push-pull help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and not __fish_seen_subcommand_from device plan run sessions push-pull help" -f -a "device" -d 'Manage sync devices'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and not __fish_seen_subcommand_from device plan run sessions push-pull help" -f -a "plan" -d 'Negotiate sync operations without modifying files'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and not __fish_seen_subcommand_from device plan run sessions push-pull help" -f -a "run" -d 'Execute one-shot sync operations'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and not __fish_seen_subcommand_from device plan run sessions push-pull help" -f -a "sessions" -d 'Inspect sync sessions'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and not __fish_seen_subcommand_from device plan run sessions push-pull help" -f -a "push-pull" -d 'Trigger push-pull mode on a registered device'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and not __fish_seen_subcommand_from device plan run sessions push-pull help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from device" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from device" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from device" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from device" -f -a "register" -d 'Register a device (`POST /api/devices`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from device" -f -a "list" -d 'List devices (`GET /api/devices`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from device" -f -a "get" -d 'Get one device (`GET /api/devices/{id}`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from device" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from plan" -l device-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from plan" -l manifest -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from plan" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from plan" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from plan" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from run" -l device-id -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from run" -l manifest -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from run" -l download-dir -d 'Folder for downloaded saves (defaults to the manifest directory)' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from run" -l conflict -r -f -a "fail\t''
skip\t''"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from run" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from run" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from run" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from sessions" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from sessions" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from sessions" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from sessions" -f -a "list" -d 'List sessions (`GET /api/sync/sessions`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from sessions" -f -a "get" -d 'Get one session (`GET /api/sync/sessions/{id}`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from sessions" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from push-pull" -l json -d 'Output as JSON (overrides global --json when set)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from push-pull" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from push-pull" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from help" -f -a "device" -d 'Manage sync devices'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from help" -f -a "plan" -d 'Negotiate sync operations without modifying files'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from help" -f -a "run" -d 'Execute one-shot sync operations'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from help" -f -a "sessions" -d 'Inspect sync sessions'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from help" -f -a "push-pull" -d 'Trigger push-pull mode on a registered device'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand sync; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -a "batch" -d 'Download multiple ROMs matching filters'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -a "all" -d 'Download multiple ROMs matching filters'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -a "extras" -d 'Download covers, manuals, updates, and DLC for one game'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and not __fish_seen_subcommand_from batch all extras help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from batch" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from all" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from extras" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from help" -f -a "batch" -d 'Download multiple ROMs matching filters'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from help" -f -a "extras" -d 'Download covers, manuals, updates, and DLC for one game'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand download; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -a "batch" -d 'Download multiple ROMs matching filters'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -a "all" -d 'Download multiple ROMs matching filters'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -a "extras" -d 'Download covers, manuals, updates, and DLC for one game'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and not __fish_seen_subcommand_from batch all extras help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from batch" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from all" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from extras" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from help" -f -a "batch" -d 'Download multiple ROMs matching filters'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from help" -f -a "extras" -d 'Download covers, manuals, updates, and DLC for one game'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand dl; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -a "batch" -d 'Download multiple ROMs matching filters'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -a "all" -d 'Download multiple ROMs matching filters'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -a "extras" -d 'Download covers, manuals, updates, and DLC for one game'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and not __fish_seen_subcommand_from batch all extras help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from batch" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from all" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -s o -l output -d 'Directory to save the ROM zip(s) to' -r -F
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -l platform -d 'Filter by platform slug or title (e.g. "3ds")' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -l search-term -d 'Filter by search term' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -l jobs -d 'Maximum concurrent downloads (default: 4)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -l extract-layout -d 'Layout for extracted files when --extract is set (default: platform)' -r -f -a "platform\t'Extract to `<output>/<platform_slug>/`'
flat\t'Extract to `<output>/`'
rom\t'Extract to `<output>/<platform_slug>/<rom_name>/`'"
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -l extract -d 'Extract each downloaded ZIP after download completes (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -l delete-zip-after-extract -d 'Delete ZIP files after successful extraction (batch mode only)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -l with-extras -d 'Include updates and DLC after downloading the base game (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -l no-extras -d 'Skip updates and DLC (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -s y -l yes -d 'Assume yes for extras prompt (single-ROM mode)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from extras" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "batch" -d 'Download multiple ROMs matching filters'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "extras" -d 'Download covers, manuals, updates, and DLC for one game'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and not __fish_seen_subcommand_from path info clear help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and not __fish_seen_subcommand_from path info clear help" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and not __fish_seen_subcommand_from path info clear help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and not __fish_seen_subcommand_from path info clear help" -f -a "path" -d 'Print the effective ROM cache file path'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and not __fish_seen_subcommand_from path info clear help" -f -a "info" -d 'Show ROM cache metadata and parse status'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and not __fish_seen_subcommand_from path info clear help" -f -a "clear" -d 'Delete the ROM cache file if it exists'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and not __fish_seen_subcommand_from path info clear help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from path" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from path" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from path" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from info" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from info" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from clear" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from clear" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from clear" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from help" -f -a "path" -d 'Print the effective ROM cache file path'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from help" -f -a "info" -d 'Show ROM cache metadata and parse status'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from help" -f -a "clear" -d 'Delete the ROM cache file if it exists'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand cache; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -f -a "login" -d 'Set/rotate authentication credentials (Bearer, Basic, API key, or pairing code)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -f -a "logout" -d 'Remove stored authentication (leaves non-auth config untouched)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -f -a "status" -d 'Show current authentication mode and where it comes from (env/config/keyring)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l token -d 'API token (Bearer). Skips interactive prompts' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l token-file -d 'Read API token (Bearer) from UTF-8 file. Use \'-\' for stdin' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l username -d 'Basic auth username' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l password -d 'Basic auth password (discouraged: visible in process list)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l password-file -d 'Read Basic auth password from a UTF-8 file. Use \'-\' for stdin' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l api-key-header -d 'API key header name (e.g. X-API-Key)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l api-key -d 'API key value' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l pairing-code -d 'Web UI pairing code (8 characters)' -r
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from login" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from logout" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from logout" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from logout" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from status" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "login" -d 'Set/rotate authentication credentials (Bearer, Basic, API key, or pairing code)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "logout" -d 'Remove stored authentication (leaves non-auth config untouched)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show current authentication mode and where it comes from (env/config/keyring)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand update" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand update" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand update" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand completions" -s v -l verbose -d 'Increase output verbosity (logs requests to stderr)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand completions" -l json -d 'Output JSON instead of human-readable text where supported'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand completions" -s h -l help -d 'Print help'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "init" -d 'Create or update user configuration'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "tui" -d 'Launch the interactive Terminal User Interface (TUI)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "api" -d 'Low-level access to any RomM API endpoint'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "platforms" -d 'Manage gaming platforms'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "roms" -d 'Manage ROM files and metadata'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "scan" -d 'Trigger a library scan on the RomM server'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "sync" -d 'Save-sync workflows (device registration, planning, and execution)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "download" -d 'Download a ROM or related extras from the server'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "cache" -d 'Manage the local persistent cache'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "auth" -d 'Manage authentication credentials'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "update" -d 'Check for and install application updates'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "completions" -d 'Generate shell completion scripts'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and not __fish_seen_subcommand_from init tui api platforms roms scan sync download cache auth update completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from api" -f -a "call" -d 'Make a generic API call'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from api" -f -a "get" -d 'Shortcut for GET request'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from api" -f -a "post" -d 'Shortcut for POST request'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from platforms" -f -a "list" -d 'List all platforms (default)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from platforms" -f -a "get" -d 'Get details for a specific platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "get" -d 'Get detailed information for a single ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "find" -d 'Lookup ROM by file hash or metadata provider id'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "filters" -d 'Print canonical filter values from `GET /api/roms/filters`'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "delete" -d 'Delete ROMs from the database (optional filesystem delete)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "props" -d 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "notes-list" -d 'List notes for a ROM'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "notes-add" -d 'Add a note (JSON body string, e.g. {\\"title\\":\\"t\\",\\"content\\":\\"c\\"})'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "notes-update" -d 'Update a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "notes-delete" -d 'Delete a note'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "manuals-add" -d 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "cover-search" -d 'Search covers and metadata matches'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from roms" -f -a "upload" -d 'Upload a ROM file to a platform'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from sync" -f -a "device" -d 'Manage sync devices'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from sync" -f -a "plan" -d 'Negotiate sync operations without modifying files'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from sync" -f -a "run" -d 'Execute one-shot sync operations'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from sync" -f -a "sessions" -d 'Inspect sync sessions'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from sync" -f -a "push-pull" -d 'Trigger push-pull mode on a registered device'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from download" -f -a "batch" -d 'Download multiple ROMs matching filters'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from download" -f -a "extras" -d 'Download covers, manuals, updates, and DLC for one game'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from cache" -f -a "path" -d 'Print the effective ROM cache file path'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from cache" -f -a "info" -d 'Show ROM cache metadata and parse status'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from cache" -f -a "clear" -d 'Delete the ROM cache file if it exists'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "login" -d 'Set/rotate authentication credentials (Bearer, Basic, API key, or pairing code)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "logout" -d 'Remove stored authentication (leaves non-auth config untouched)'
complete -c romm-cli -n "__fish_romm_cli_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "status" -d 'Show current authentication mode and where it comes from (env/config/keyring)'
