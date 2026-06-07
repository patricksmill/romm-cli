
use builtin;
use str;

set edit:completion:arg-completer[romm-cli] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'romm-cli'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'romm-cli'= {
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
            cand init 'Create or update user configuration'
            cand setup 'Create or update user configuration'
            cand tui 'Launch the interactive Terminal User Interface (TUI)'
            cand api 'Low-level access to any RomM API endpoint'
            cand call 'Low-level access to any RomM API endpoint'
            cand platforms 'Manage gaming platforms'
            cand platform 'Manage gaming platforms'
            cand p 'Manage gaming platforms'
            cand plats 'Manage gaming platforms'
            cand roms 'Manage ROM files and metadata'
            cand rom 'Manage ROM files and metadata'
            cand r 'Manage ROM files and metadata'
            cand scan 'Trigger a library scan on the RomM server'
            cand sync 'Save-sync workflows (device registration, planning, and execution)'
            cand download 'Download a ROM or related extras from the server'
            cand dl 'Download a ROM or related extras from the server'
            cand get 'Download a ROM or related extras from the server'
            cand cache 'Manage the local persistent cache'
            cand auth 'Manage authentication credentials'
            cand update 'Check for and install application updates'
            cand completions 'Generate shell completion scripts'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;init'= {
            cand --url 'RomM origin URL (e.g. <https://romm.example>). If provided with a token, skips interactive prompts'
            cand --token 'Bearer token string (discouraged: visible in process list)'
            cand --token-file 'Read Bearer token from a UTF-8 file. Use ''-'' for stdin'
            cand --download-dir 'ROMs directory'
            cand --force 'Overwrite existing user config `config.json` without asking'
            cand --print-path 'Print the path to the user config `config.json` and exit'
            cand --no-https 'Disable HTTPS (use HTTP instead)'
            cand --check 'Verify URL and token by fetching OpenAPI after saving'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;setup'= {
            cand --url 'RomM origin URL (e.g. <https://romm.example>). If provided with a token, skips interactive prompts'
            cand --token 'Bearer token string (discouraged: visible in process list)'
            cand --token-file 'Read Bearer token from a UTF-8 file. Use ''-'' for stdin'
            cand --download-dir 'ROMs directory'
            cand --force 'Overwrite existing user config `config.json` without asking'
            cand --print-path 'Print the path to the user config `config.json` and exit'
            cand --no-https 'Disable HTTPS (use HTTP instead)'
            cand --check 'Verify URL and token by fetching OpenAPI after saving'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;tui'= {
            cand --mock-update 'Force show a fake update prompt for UI testing'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;api'= {
            cand --query 'Query parameters as key=value, repeatable'
            cand --data 'JSON request body as a string'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
            cand call 'Make a generic API call'
            cand get 'Shortcut for GET request'
            cand post 'Shortcut for POST request'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;call'= {
            cand --query 'Query parameters as key=value, repeatable'
            cand --data 'JSON request body as a string'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
            cand call 'Make a generic API call'
            cand get 'Shortcut for GET request'
            cand post 'Shortcut for POST request'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;api;call'= {
            cand --query 'Query parameters as key=value, repeatable'
            cand --data 'JSON request body as a string'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;call;call'= {
            cand --query 'Query parameters as key=value, repeatable'
            cand --data 'JSON request body as a string'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;api;get'= {
            cand --query 'Query parameters as key=value, repeatable'
            cand --data 'JSON request body as a string'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;call;get'= {
            cand --query 'Query parameters as key=value, repeatable'
            cand --data 'JSON request body as a string'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;api;post'= {
            cand --query 'Query parameters as key=value, repeatable'
            cand --data 'JSON request body as a string'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;call;post'= {
            cand --query 'Query parameters as key=value, repeatable'
            cand --data 'JSON request body as a string'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;api;help'= {
            cand call 'Make a generic API call'
            cand get 'Shortcut for GET request'
            cand post 'Shortcut for POST request'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;api;help;call'= {
        }
        &'romm-cli;api;help;get'= {
        }
        &'romm-cli;api;help;post'= {
        }
        &'romm-cli;api;help;help'= {
        }
        &'romm-cli;call;help'= {
            cand call 'Make a generic API call'
            cand get 'Shortcut for GET request'
            cand post 'Shortcut for POST request'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;call;help;call'= {
        }
        &'romm-cli;call;help;get'= {
        }
        &'romm-cli;call;help;post'= {
        }
        &'romm-cli;call;help;help'= {
        }
        &'romm-cli;platforms'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand list 'List all platforms (default)'
            cand ls 'List all platforms (default)'
            cand get 'Get details for a specific platform'
            cand info 'Get details for a specific platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;platform'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand list 'List all platforms (default)'
            cand ls 'List all platforms (default)'
            cand get 'Get details for a specific platform'
            cand info 'Get details for a specific platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;p'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand list 'List all platforms (default)'
            cand ls 'List all platforms (default)'
            cand get 'Get details for a specific platform'
            cand info 'Get details for a specific platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;plats'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand list 'List all platforms (default)'
            cand ls 'List all platforms (default)'
            cand get 'Get details for a specific platform'
            cand info 'Get details for a specific platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;platforms;list'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;platforms;ls'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;platform;list'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;platform;ls'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;p;list'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;p;ls'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;plats;list'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;plats;ls'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;platforms;get'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;platforms;info'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;platform;get'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;platform;info'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;p;get'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;p;info'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;plats;get'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;plats;info'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;platforms;help'= {
            cand list 'List all platforms (default)'
            cand get 'Get details for a specific platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;platforms;help;list'= {
        }
        &'romm-cli;platforms;help;get'= {
        }
        &'romm-cli;platforms;help;help'= {
        }
        &'romm-cli;platform;help'= {
            cand list 'List all platforms (default)'
            cand get 'Get details for a specific platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;platform;help;list'= {
        }
        &'romm-cli;platform;help;get'= {
        }
        &'romm-cli;platform;help;help'= {
        }
        &'romm-cli;p;help'= {
            cand list 'List all platforms (default)'
            cand get 'Get details for a specific platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;p;help;list'= {
        }
        &'romm-cli;p;help;get'= {
        }
        &'romm-cli;p;help;help'= {
        }
        &'romm-cli;plats;help'= {
            cand list 'List all platforms (default)'
            cand get 'Get details for a specific platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;plats;help;list'= {
        }
        &'romm-cli;plats;help;get'= {
        }
        &'romm-cli;plats;help;help'= {
        }
        &'romm-cli;roms'= {
            cand --search-term 'search-term'
            cand --query 'search-term'
            cand --q 'search-term'
            cand --platform 'Platform slug or name; repeat for multiple `platform_ids`'
            cand --p 'Platform slug or name; repeat for multiple `platform_ids`'
            cand --collection 'Manual collection id or exact name'
            cand --smart-collection 'Smart collection id or exact name'
            cand --virtual-collection 'Virtual collection id (e.g. recent)'
            cand --limit 'limit'
            cand --offset 'offset'
            cand --matched 'matched'
            cand --favorite 'favorite'
            cand --duplicate 'duplicate'
            cand --last-played 'last-played'
            cand --playable 'playable'
            cand --missing 'missing'
            cand --has-ra 'has-ra'
            cand --verified 'verified'
            cand --group-by-meta-id 'group-by-meta-id'
            cand --with-char-index 'with-char-index'
            cand --with-filter-values 'with-filter-values'
            cand --genre 'genre'
            cand --franchise 'franchise'
            cand --collection-tag 'collection-tag'
            cand --company 'company'
            cand --age-rating 'age-rating'
            cand --status 'status'
            cand --region 'region'
            cand --language 'language'
            cand --player-count 'player-count'
            cand --genres-logic 'genres-logic'
            cand --franchises-logic 'franchises-logic'
            cand --collections-logic 'collections-logic'
            cand --companies-logic 'companies-logic'
            cand --age-ratings-logic 'age-ratings-logic'
            cand --regions-logic 'regions-logic'
            cand --languages-logic 'languages-logic'
            cand --statuses-logic 'statuses-logic'
            cand --player-counts-logic 'player-counts-logic'
            cand --order-by 'order-by'
            cand --order-dir 'order-dir'
            cand --updated-after 'updated-after'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand get 'Get detailed information for a single ROM'
            cand info 'Get detailed information for a single ROM'
            cand find 'Lookup ROM by file hash or metadata provider id'
            cand filters 'Print canonical filter values from `GET /api/roms/filters`'
            cand delete 'Delete ROMs from the database (optional filesystem delete)'
            cand props 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
            cand notes-list 'List notes for a ROM'
            cand notes-add 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})'
            cand notes-update 'Update a note'
            cand notes-delete 'Delete a note'
            cand manuals-add 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
            cand cover-search 'Search covers and metadata matches'
            cand upload 'Upload a ROM file to a platform'
            cand up 'Upload a ROM file to a platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;rom'= {
            cand --search-term 'search-term'
            cand --query 'search-term'
            cand --q 'search-term'
            cand --platform 'Platform slug or name; repeat for multiple `platform_ids`'
            cand --p 'Platform slug or name; repeat for multiple `platform_ids`'
            cand --collection 'Manual collection id or exact name'
            cand --smart-collection 'Smart collection id or exact name'
            cand --virtual-collection 'Virtual collection id (e.g. recent)'
            cand --limit 'limit'
            cand --offset 'offset'
            cand --matched 'matched'
            cand --favorite 'favorite'
            cand --duplicate 'duplicate'
            cand --last-played 'last-played'
            cand --playable 'playable'
            cand --missing 'missing'
            cand --has-ra 'has-ra'
            cand --verified 'verified'
            cand --group-by-meta-id 'group-by-meta-id'
            cand --with-char-index 'with-char-index'
            cand --with-filter-values 'with-filter-values'
            cand --genre 'genre'
            cand --franchise 'franchise'
            cand --collection-tag 'collection-tag'
            cand --company 'company'
            cand --age-rating 'age-rating'
            cand --status 'status'
            cand --region 'region'
            cand --language 'language'
            cand --player-count 'player-count'
            cand --genres-logic 'genres-logic'
            cand --franchises-logic 'franchises-logic'
            cand --collections-logic 'collections-logic'
            cand --companies-logic 'companies-logic'
            cand --age-ratings-logic 'age-ratings-logic'
            cand --regions-logic 'regions-logic'
            cand --languages-logic 'languages-logic'
            cand --statuses-logic 'statuses-logic'
            cand --player-counts-logic 'player-counts-logic'
            cand --order-by 'order-by'
            cand --order-dir 'order-dir'
            cand --updated-after 'updated-after'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand get 'Get detailed information for a single ROM'
            cand info 'Get detailed information for a single ROM'
            cand find 'Lookup ROM by file hash or metadata provider id'
            cand filters 'Print canonical filter values from `GET /api/roms/filters`'
            cand delete 'Delete ROMs from the database (optional filesystem delete)'
            cand props 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
            cand notes-list 'List notes for a ROM'
            cand notes-add 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})'
            cand notes-update 'Update a note'
            cand notes-delete 'Delete a note'
            cand manuals-add 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
            cand cover-search 'Search covers and metadata matches'
            cand upload 'Upload a ROM file to a platform'
            cand up 'Upload a ROM file to a platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;r'= {
            cand --search-term 'search-term'
            cand --query 'search-term'
            cand --q 'search-term'
            cand --platform 'Platform slug or name; repeat for multiple `platform_ids`'
            cand --p 'Platform slug or name; repeat for multiple `platform_ids`'
            cand --collection 'Manual collection id or exact name'
            cand --smart-collection 'Smart collection id or exact name'
            cand --virtual-collection 'Virtual collection id (e.g. recent)'
            cand --limit 'limit'
            cand --offset 'offset'
            cand --matched 'matched'
            cand --favorite 'favorite'
            cand --duplicate 'duplicate'
            cand --last-played 'last-played'
            cand --playable 'playable'
            cand --missing 'missing'
            cand --has-ra 'has-ra'
            cand --verified 'verified'
            cand --group-by-meta-id 'group-by-meta-id'
            cand --with-char-index 'with-char-index'
            cand --with-filter-values 'with-filter-values'
            cand --genre 'genre'
            cand --franchise 'franchise'
            cand --collection-tag 'collection-tag'
            cand --company 'company'
            cand --age-rating 'age-rating'
            cand --status 'status'
            cand --region 'region'
            cand --language 'language'
            cand --player-count 'player-count'
            cand --genres-logic 'genres-logic'
            cand --franchises-logic 'franchises-logic'
            cand --collections-logic 'collections-logic'
            cand --companies-logic 'companies-logic'
            cand --age-ratings-logic 'age-ratings-logic'
            cand --regions-logic 'regions-logic'
            cand --languages-logic 'languages-logic'
            cand --statuses-logic 'statuses-logic'
            cand --player-counts-logic 'player-counts-logic'
            cand --order-by 'order-by'
            cand --order-dir 'order-dir'
            cand --updated-after 'updated-after'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand get 'Get detailed information for a single ROM'
            cand info 'Get detailed information for a single ROM'
            cand find 'Lookup ROM by file hash or metadata provider id'
            cand filters 'Print canonical filter values from `GET /api/roms/filters`'
            cand delete 'Delete ROMs from the database (optional filesystem delete)'
            cand props 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
            cand notes-list 'List notes for a ROM'
            cand notes-add 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})'
            cand notes-update 'Update a note'
            cand notes-delete 'Delete a note'
            cand manuals-add 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
            cand cover-search 'Search covers and metadata matches'
            cand upload 'Upload a ROM file to a platform'
            cand up 'Upload a ROM file to a platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;roms;get'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;info'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;get'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;info'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;get'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;info'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;find'= {
            cand --crc 'crc'
            cand --md5 'md5'
            cand --sha1 'sha1'
            cand --igdb-id 'igdb-id'
            cand --moby-id 'moby-id'
            cand --ss-id 'ss-id'
            cand --ra-id 'ra-id'
            cand --launchbox-id 'launchbox-id'
            cand --hasheous-id 'hasheous-id'
            cand --tgdb-id 'tgdb-id'
            cand --flashpoint-id 'flashpoint-id'
            cand --hltb-id 'hltb-id'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;find'= {
            cand --crc 'crc'
            cand --md5 'md5'
            cand --sha1 'sha1'
            cand --igdb-id 'igdb-id'
            cand --moby-id 'moby-id'
            cand --ss-id 'ss-id'
            cand --ra-id 'ra-id'
            cand --launchbox-id 'launchbox-id'
            cand --hasheous-id 'hasheous-id'
            cand --tgdb-id 'tgdb-id'
            cand --flashpoint-id 'flashpoint-id'
            cand --hltb-id 'hltb-id'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;find'= {
            cand --crc 'crc'
            cand --md5 'md5'
            cand --sha1 'sha1'
            cand --igdb-id 'igdb-id'
            cand --moby-id 'moby-id'
            cand --ss-id 'ss-id'
            cand --ra-id 'ra-id'
            cand --launchbox-id 'launchbox-id'
            cand --hasheous-id 'hasheous-id'
            cand --tgdb-id 'tgdb-id'
            cand --flashpoint-id 'flashpoint-id'
            cand --hltb-id 'hltb-id'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;filters'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;filters'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;filters'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;delete'= {
            cand --delete-from-fs 'Also delete these ROM ids from disk (repeat ids as needed)'
            cand --yes 'Skip confirmation'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;delete'= {
            cand --delete-from-fs 'Also delete these ROM ids from disk (repeat ids as needed)'
            cand --yes 'Skip confirmation'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;delete'= {
            cand --delete-from-fs 'Also delete these ROM ids from disk (repeat ids as needed)'
            cand --yes 'Skip confirmation'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;props'= {
            cand --is-main-sibling 'is-main-sibling'
            cand --backlogged 'backlogged'
            cand --now-playing 'now-playing'
            cand --hidden 'hidden'
            cand --rating 'rating'
            cand --difficulty 'difficulty'
            cand --completion 'completion'
            cand --status 'status'
            cand --update-last-played 'update-last-played'
            cand --remove-last-played 'remove-last-played'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;props'= {
            cand --is-main-sibling 'is-main-sibling'
            cand --backlogged 'backlogged'
            cand --now-playing 'now-playing'
            cand --hidden 'hidden'
            cand --rating 'rating'
            cand --difficulty 'difficulty'
            cand --completion 'completion'
            cand --status 'status'
            cand --update-last-played 'update-last-played'
            cand --remove-last-played 'remove-last-played'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;props'= {
            cand --is-main-sibling 'is-main-sibling'
            cand --backlogged 'backlogged'
            cand --now-playing 'now-playing'
            cand --hidden 'hidden'
            cand --rating 'rating'
            cand --difficulty 'difficulty'
            cand --completion 'completion'
            cand --status 'status'
            cand --update-last-played 'update-last-played'
            cand --remove-last-played 'remove-last-played'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;notes-list'= {
            cand --public-only 'public-only'
            cand --search 'search'
            cand --tag 'tag'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;notes-list'= {
            cand --public-only 'public-only'
            cand --search 'search'
            cand --tag 'tag'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;notes-list'= {
            cand --public-only 'public-only'
            cand --search 'search'
            cand --tag 'tag'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;notes-add'= {
            cand --json 'JSON object'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;notes-add'= {
            cand --json 'JSON object'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;notes-add'= {
            cand --json 'JSON object'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;notes-update'= {
            cand --json 'json'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;notes-update'= {
            cand --json 'json'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;notes-update'= {
            cand --json 'json'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;notes-delete'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;notes-delete'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;notes-delete'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;manuals-add'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;manuals-add'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;manuals-add'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;cover-search'= {
            cand --query 'query'
            cand --search-by 'search-by'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;cover-search'= {
            cand --query 'query'
            cand --search-by 'search-by'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;cover-search'= {
            cand --query 'query'
            cand --search-by 'search-by'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;upload'= {
            cand --platform 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")'
            cand --wait-timeout-secs 'Max seconds to wait when `--wait` is set (default: 3600)'
            cand -s 'Trigger a library scan after upload completes'
            cand --scan 'Trigger a library scan after upload completes'
            cand --wait 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;up'= {
            cand --platform 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")'
            cand --wait-timeout-secs 'Max seconds to wait when `--wait` is set (default: 3600)'
            cand -s 'Trigger a library scan after upload completes'
            cand --scan 'Trigger a library scan after upload completes'
            cand --wait 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;upload'= {
            cand --platform 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")'
            cand --wait-timeout-secs 'Max seconds to wait when `--wait` is set (default: 3600)'
            cand -s 'Trigger a library scan after upload completes'
            cand --scan 'Trigger a library scan after upload completes'
            cand --wait 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;rom;up'= {
            cand --platform 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")'
            cand --wait-timeout-secs 'Max seconds to wait when `--wait` is set (default: 3600)'
            cand -s 'Trigger a library scan after upload completes'
            cand --scan 'Trigger a library scan after upload completes'
            cand --wait 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;upload'= {
            cand --platform 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")'
            cand --wait-timeout-secs 'Max seconds to wait when `--wait` is set (default: 3600)'
            cand -s 'Trigger a library scan after upload completes'
            cand --scan 'Trigger a library scan after upload completes'
            cand --wait 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;r;up'= {
            cand --platform 'Platform slug or name (e.g. "3ds", "Nintendo 3DS")'
            cand --wait-timeout-secs 'Max seconds to wait when `--wait` is set (default: 3600)'
            cand -s 'Trigger a library scan after upload completes'
            cand --scan 'Trigger a library scan after upload completes'
            cand --wait 'Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;roms;help'= {
            cand get 'Get detailed information for a single ROM'
            cand find 'Lookup ROM by file hash or metadata provider id'
            cand filters 'Print canonical filter values from `GET /api/roms/filters`'
            cand delete 'Delete ROMs from the database (optional filesystem delete)'
            cand props 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
            cand notes-list 'List notes for a ROM'
            cand notes-add 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})'
            cand notes-update 'Update a note'
            cand notes-delete 'Delete a note'
            cand manuals-add 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
            cand cover-search 'Search covers and metadata matches'
            cand upload 'Upload a ROM file to a platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;roms;help;get'= {
        }
        &'romm-cli;roms;help;find'= {
        }
        &'romm-cli;roms;help;filters'= {
        }
        &'romm-cli;roms;help;delete'= {
        }
        &'romm-cli;roms;help;props'= {
        }
        &'romm-cli;roms;help;notes-list'= {
        }
        &'romm-cli;roms;help;notes-add'= {
        }
        &'romm-cli;roms;help;notes-update'= {
        }
        &'romm-cli;roms;help;notes-delete'= {
        }
        &'romm-cli;roms;help;manuals-add'= {
        }
        &'romm-cli;roms;help;cover-search'= {
        }
        &'romm-cli;roms;help;upload'= {
        }
        &'romm-cli;roms;help;help'= {
        }
        &'romm-cli;rom;help'= {
            cand get 'Get detailed information for a single ROM'
            cand find 'Lookup ROM by file hash or metadata provider id'
            cand filters 'Print canonical filter values from `GET /api/roms/filters`'
            cand delete 'Delete ROMs from the database (optional filesystem delete)'
            cand props 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
            cand notes-list 'List notes for a ROM'
            cand notes-add 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})'
            cand notes-update 'Update a note'
            cand notes-delete 'Delete a note'
            cand manuals-add 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
            cand cover-search 'Search covers and metadata matches'
            cand upload 'Upload a ROM file to a platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;rom;help;get'= {
        }
        &'romm-cli;rom;help;find'= {
        }
        &'romm-cli;rom;help;filters'= {
        }
        &'romm-cli;rom;help;delete'= {
        }
        &'romm-cli;rom;help;props'= {
        }
        &'romm-cli;rom;help;notes-list'= {
        }
        &'romm-cli;rom;help;notes-add'= {
        }
        &'romm-cli;rom;help;notes-update'= {
        }
        &'romm-cli;rom;help;notes-delete'= {
        }
        &'romm-cli;rom;help;manuals-add'= {
        }
        &'romm-cli;rom;help;cover-search'= {
        }
        &'romm-cli;rom;help;upload'= {
        }
        &'romm-cli;rom;help;help'= {
        }
        &'romm-cli;r;help'= {
            cand get 'Get detailed information for a single ROM'
            cand find 'Lookup ROM by file hash or metadata provider id'
            cand filters 'Print canonical filter values from `GET /api/roms/filters`'
            cand delete 'Delete ROMs from the database (optional filesystem delete)'
            cand props 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
            cand notes-list 'List notes for a ROM'
            cand notes-add 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})'
            cand notes-update 'Update a note'
            cand notes-delete 'Delete a note'
            cand manuals-add 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
            cand cover-search 'Search covers and metadata matches'
            cand upload 'Upload a ROM file to a platform'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;r;help;get'= {
        }
        &'romm-cli;r;help;find'= {
        }
        &'romm-cli;r;help;filters'= {
        }
        &'romm-cli;r;help;delete'= {
        }
        &'romm-cli;r;help;props'= {
        }
        &'romm-cli;r;help;notes-list'= {
        }
        &'romm-cli;r;help;notes-add'= {
        }
        &'romm-cli;r;help;notes-update'= {
        }
        &'romm-cli;r;help;notes-delete'= {
        }
        &'romm-cli;r;help;manuals-add'= {
        }
        &'romm-cli;r;help;cover-search'= {
        }
        &'romm-cli;r;help;upload'= {
        }
        &'romm-cli;r;help;help'= {
        }
        &'romm-cli;scan'= {
            cand --platform 'Restrict scan to one or more platform slugs (comma-separated); passed as `platform_slugs` task kwargs'
            cand --wait-timeout-secs 'Max seconds to wait when `--wait` is set (default: 3600)'
            cand --wait 'Wait until the scan task completes (polls every 2 seconds)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;sync'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand device 'Manage sync devices'
            cand plan 'Negotiate sync operations without modifying files'
            cand run 'Execute one-shot sync operations'
            cand sessions 'Inspect sync sessions'
            cand push-pull 'Trigger push-pull mode on a registered device'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;sync;device'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand register 'Register a device (`POST /api/devices`)'
            cand list 'List devices (`GET /api/devices`)'
            cand get 'Get one device (`GET /api/devices/{id}`)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;sync;device;register'= {
            cand --name 'name'
            cand --platform 'platform'
            cand --client 'client'
            cand --client-version 'client-version'
            cand --hostname 'hostname'
            cand --mac-address 'mac-address'
            cand --ip-address 'ip-address'
            cand --sync-mode 'sync-mode'
            cand --sync-config-json 'Optional JSON object string for `sync_config`'
            cand --allow-duplicate 'allow-duplicate'
            cand --reset-syncs 'reset-syncs'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;sync;device;list'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;sync;device;get'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;sync;device;help'= {
            cand register 'Register a device (`POST /api/devices`)'
            cand list 'List devices (`GET /api/devices`)'
            cand get 'Get one device (`GET /api/devices/{id}`)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;sync;device;help;register'= {
        }
        &'romm-cli;sync;device;help;list'= {
        }
        &'romm-cli;sync;device;help;get'= {
        }
        &'romm-cli;sync;device;help;help'= {
        }
        &'romm-cli;sync;plan'= {
            cand --device-id 'device-id'
            cand --manifest 'manifest'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;sync;run'= {
            cand --device-id 'device-id'
            cand --manifest 'manifest'
            cand --download-dir 'Folder for downloaded saves (defaults to the manifest directory)'
            cand --conflict 'conflict'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;sync;sessions'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand list 'List sessions (`GET /api/sync/sessions`)'
            cand get 'Get one session (`GET /api/sync/sessions/{id}`)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;sync;sessions;list'= {
            cand --device-id 'device-id'
            cand --limit 'limit'
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;sync;sessions;get'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;sync;sessions;help'= {
            cand list 'List sessions (`GET /api/sync/sessions`)'
            cand get 'Get one session (`GET /api/sync/sessions/{id}`)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;sync;sessions;help;list'= {
        }
        &'romm-cli;sync;sessions;help;get'= {
        }
        &'romm-cli;sync;sessions;help;help'= {
        }
        &'romm-cli;sync;push-pull'= {
            cand --json 'Output as JSON (overrides global --json when set)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;sync;help'= {
            cand device 'Manage sync devices'
            cand plan 'Negotiate sync operations without modifying files'
            cand run 'Execute one-shot sync operations'
            cand sessions 'Inspect sync sessions'
            cand push-pull 'Trigger push-pull mode on a registered device'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;sync;help;device'= {
            cand register 'Register a device (`POST /api/devices`)'
            cand list 'List devices (`GET /api/devices`)'
            cand get 'Get one device (`GET /api/devices/{id}`)'
        }
        &'romm-cli;sync;help;device;register'= {
        }
        &'romm-cli;sync;help;device;list'= {
        }
        &'romm-cli;sync;help;device;get'= {
        }
        &'romm-cli;sync;help;plan'= {
        }
        &'romm-cli;sync;help;run'= {
        }
        &'romm-cli;sync;help;sessions'= {
            cand list 'List sessions (`GET /api/sync/sessions`)'
            cand get 'Get one session (`GET /api/sync/sessions/{id}`)'
        }
        &'romm-cli;sync;help;sessions;list'= {
        }
        &'romm-cli;sync;help;sessions;get'= {
        }
        &'romm-cli;sync;help;push-pull'= {
        }
        &'romm-cli;sync;help;help'= {
        }
        &'romm-cli;download'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand batch 'Download multiple ROMs matching filters'
            cand all 'Download multiple ROMs matching filters'
            cand extras 'Download covers, manuals, updates, and DLC for one game'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;dl'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand batch 'Download multiple ROMs matching filters'
            cand all 'Download multiple ROMs matching filters'
            cand extras 'Download covers, manuals, updates, and DLC for one game'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;get'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand batch 'Download multiple ROMs matching filters'
            cand all 'Download multiple ROMs matching filters'
            cand extras 'Download covers, manuals, updates, and DLC for one game'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;download;batch'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'romm-cli;download;all'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'romm-cli;dl;batch'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'romm-cli;dl;all'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'romm-cli;get;batch'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'romm-cli;get;all'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'romm-cli;download;extras'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'romm-cli;dl;extras'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'romm-cli;get;extras'= {
            cand -o 'Directory to save the ROM zip(s) to'
            cand --output 'Directory to save the ROM zip(s) to'
            cand --platform 'Filter by platform slug or title (e.g. "3ds")'
            cand --search-term 'Filter by search term'
            cand --jobs 'Maximum concurrent downloads (default: 4)'
            cand --extract-layout 'Layout for extracted files when --extract is set (default: platform)'
            cand --extract 'Extract each downloaded ZIP after download completes (batch mode only)'
            cand --delete-zip-after-extract 'Delete ZIP files after successful extraction (batch mode only)'
            cand --with-extras 'Include updates and DLC after downloading the base game (single-ROM mode)'
            cand --no-extras 'Skip updates and DLC (single-ROM mode)'
            cand -y 'Assume yes for extras prompt (single-ROM mode)'
            cand --yes 'Assume yes for extras prompt (single-ROM mode)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'romm-cli;download;help'= {
            cand batch 'Download multiple ROMs matching filters'
            cand extras 'Download covers, manuals, updates, and DLC for one game'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;download;help;batch'= {
        }
        &'romm-cli;download;help;extras'= {
        }
        &'romm-cli;download;help;help'= {
        }
        &'romm-cli;dl;help'= {
            cand batch 'Download multiple ROMs matching filters'
            cand extras 'Download covers, manuals, updates, and DLC for one game'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;dl;help;batch'= {
        }
        &'romm-cli;dl;help;extras'= {
        }
        &'romm-cli;dl;help;help'= {
        }
        &'romm-cli;get;help'= {
            cand batch 'Download multiple ROMs matching filters'
            cand extras 'Download covers, manuals, updates, and DLC for one game'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;get;help;batch'= {
        }
        &'romm-cli;get;help;extras'= {
        }
        &'romm-cli;get;help;help'= {
        }
        &'romm-cli;cache'= {
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
            cand path 'Print the effective ROM cache file path'
            cand info 'Show ROM cache metadata and parse status'
            cand clear 'Delete the ROM cache file if it exists'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;cache;path'= {
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;cache;info'= {
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;cache;clear'= {
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;cache;help'= {
            cand path 'Print the effective ROM cache file path'
            cand info 'Show ROM cache metadata and parse status'
            cand clear 'Delete the ROM cache file if it exists'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;cache;help;path'= {
        }
        &'romm-cli;cache;help;info'= {
        }
        &'romm-cli;cache;help;clear'= {
        }
        &'romm-cli;cache;help;help'= {
        }
        &'romm-cli;auth'= {
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
            cand login 'Set/rotate authentication credentials (Bearer, Basic, API key, or pairing code)'
            cand logout 'Remove stored authentication (leaves non-auth config untouched)'
            cand status 'Show current authentication mode and where it comes from (env/config/keyring)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;auth;login'= {
            cand --token 'API token (Bearer). Skips interactive prompts'
            cand --token-file 'Read API token (Bearer) from UTF-8 file. Use ''-'' for stdin'
            cand --username 'Basic auth username'
            cand --password 'Basic auth password (discouraged: visible in process list)'
            cand --password-file 'Read Basic auth password from a UTF-8 file. Use ''-'' for stdin'
            cand --api-key-header 'API key header name (e.g. X-API-Key)'
            cand --api-key 'API key value'
            cand --pairing-code 'Web UI pairing code (8 characters)'
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;auth;logout'= {
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;auth;status'= {
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;auth;help'= {
            cand login 'Set/rotate authentication credentials (Bearer, Basic, API key, or pairing code)'
            cand logout 'Remove stored authentication (leaves non-auth config untouched)'
            cand status 'Show current authentication mode and where it comes from (env/config/keyring)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;auth;help;login'= {
        }
        &'romm-cli;auth;help;logout'= {
        }
        &'romm-cli;auth;help;status'= {
        }
        &'romm-cli;auth;help;help'= {
        }
        &'romm-cli;update'= {
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;completions'= {
            cand -v 'Increase output verbosity (logs requests to stderr)'
            cand --verbose 'Increase output verbosity (logs requests to stderr)'
            cand --json 'Output JSON instead of human-readable text where supported'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'romm-cli;help'= {
            cand init 'Create or update user configuration'
            cand tui 'Launch the interactive Terminal User Interface (TUI)'
            cand api 'Low-level access to any RomM API endpoint'
            cand platforms 'Manage gaming platforms'
            cand roms 'Manage ROM files and metadata'
            cand scan 'Trigger a library scan on the RomM server'
            cand sync 'Save-sync workflows (device registration, planning, and execution)'
            cand download 'Download a ROM or related extras from the server'
            cand cache 'Manage the local persistent cache'
            cand auth 'Manage authentication credentials'
            cand update 'Check for and install application updates'
            cand completions 'Generate shell completion scripts'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'romm-cli;help;init'= {
        }
        &'romm-cli;help;tui'= {
        }
        &'romm-cli;help;api'= {
            cand call 'Make a generic API call'
            cand get 'Shortcut for GET request'
            cand post 'Shortcut for POST request'
        }
        &'romm-cli;help;api;call'= {
        }
        &'romm-cli;help;api;get'= {
        }
        &'romm-cli;help;api;post'= {
        }
        &'romm-cli;help;platforms'= {
            cand list 'List all platforms (default)'
            cand get 'Get details for a specific platform'
        }
        &'romm-cli;help;platforms;list'= {
        }
        &'romm-cli;help;platforms;get'= {
        }
        &'romm-cli;help;roms'= {
            cand get 'Get detailed information for a single ROM'
            cand find 'Lookup ROM by file hash or metadata provider id'
            cand filters 'Print canonical filter values from `GET /api/roms/filters`'
            cand delete 'Delete ROMs from the database (optional filesystem delete)'
            cand props 'Update per-user ROM properties (`PUT /api/roms/{id}/props`)'
            cand notes-list 'List notes for a ROM'
            cand notes-add 'Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})'
            cand notes-update 'Update a note'
            cand notes-delete 'Delete a note'
            cand manuals-add 'Upload a manual file (`POST /api/roms/{id}/manuals`)'
            cand cover-search 'Search covers and metadata matches'
            cand upload 'Upload a ROM file to a platform'
        }
        &'romm-cli;help;roms;get'= {
        }
        &'romm-cli;help;roms;find'= {
        }
        &'romm-cli;help;roms;filters'= {
        }
        &'romm-cli;help;roms;delete'= {
        }
        &'romm-cli;help;roms;props'= {
        }
        &'romm-cli;help;roms;notes-list'= {
        }
        &'romm-cli;help;roms;notes-add'= {
        }
        &'romm-cli;help;roms;notes-update'= {
        }
        &'romm-cli;help;roms;notes-delete'= {
        }
        &'romm-cli;help;roms;manuals-add'= {
        }
        &'romm-cli;help;roms;cover-search'= {
        }
        &'romm-cli;help;roms;upload'= {
        }
        &'romm-cli;help;scan'= {
        }
        &'romm-cli;help;sync'= {
            cand device 'Manage sync devices'
            cand plan 'Negotiate sync operations without modifying files'
            cand run 'Execute one-shot sync operations'
            cand sessions 'Inspect sync sessions'
            cand push-pull 'Trigger push-pull mode on a registered device'
        }
        &'romm-cli;help;sync;device'= {
            cand register 'Register a device (`POST /api/devices`)'
            cand list 'List devices (`GET /api/devices`)'
            cand get 'Get one device (`GET /api/devices/{id}`)'
        }
        &'romm-cli;help;sync;device;register'= {
        }
        &'romm-cli;help;sync;device;list'= {
        }
        &'romm-cli;help;sync;device;get'= {
        }
        &'romm-cli;help;sync;plan'= {
        }
        &'romm-cli;help;sync;run'= {
        }
        &'romm-cli;help;sync;sessions'= {
            cand list 'List sessions (`GET /api/sync/sessions`)'
            cand get 'Get one session (`GET /api/sync/sessions/{id}`)'
        }
        &'romm-cli;help;sync;sessions;list'= {
        }
        &'romm-cli;help;sync;sessions;get'= {
        }
        &'romm-cli;help;sync;push-pull'= {
        }
        &'romm-cli;help;download'= {
            cand batch 'Download multiple ROMs matching filters'
            cand extras 'Download covers, manuals, updates, and DLC for one game'
        }
        &'romm-cli;help;download;batch'= {
        }
        &'romm-cli;help;download;extras'= {
        }
        &'romm-cli;help;cache'= {
            cand path 'Print the effective ROM cache file path'
            cand info 'Show ROM cache metadata and parse status'
            cand clear 'Delete the ROM cache file if it exists'
        }
        &'romm-cli;help;cache;path'= {
        }
        &'romm-cli;help;cache;info'= {
        }
        &'romm-cli;help;cache;clear'= {
        }
        &'romm-cli;help;auth'= {
            cand login 'Set/rotate authentication credentials (Bearer, Basic, API key, or pairing code)'
            cand logout 'Remove stored authentication (leaves non-auth config untouched)'
            cand status 'Show current authentication mode and where it comes from (env/config/keyring)'
        }
        &'romm-cli;help;auth;login'= {
        }
        &'romm-cli;help;auth;logout'= {
        }
        &'romm-cli;help;auth;status'= {
        }
        &'romm-cli;help;update'= {
        }
        &'romm-cli;help;completions'= {
        }
        &'romm-cli;help;help'= {
        }
    ]
    $completions[$command]
}
