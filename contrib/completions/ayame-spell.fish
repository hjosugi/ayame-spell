# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_ayame_spell_global_optspecs
	string join \n config= no-config no-baseline mode= exclude= no-ignore hidden color= q/quiet v/verbose stdin-filename= max-file-size= j/threads= no-cache cache-dir= w/write format= list-rules lang= h/help V/version
end

function __fish_ayame_spell_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_ayame_spell_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_ayame_spell_using_subcommand
	set -l cmd (__fish_ayame_spell_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l config -d 'Load exactly this configuration file' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l mode -d 'Override `[check].mode`' -r -f -a "corrections\t''
dictionary\t''
off\t''"
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l exclude -d 'Exclude an additional glob (repeatable)' -r
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l color -d 'Colour policy for human output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l stdin-filename -d 'Display name used for standard input (also selects overrides)' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l max-file-size -d 'Skip files larger than this many bytes (overrides `[files].max-file-size`)' -r
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -s j -l threads -d 'Worker threads (overrides the detected CPU count)' -r
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l cache-dir -d 'Use this incremental cache directory (also enables caching in CI)' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l format -d 'Output format' -r -f -a "human\t''
brief\t''
json\t''
github\t''
sarif\t''"
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l lang -d 'Language for `--list-rules` (defaults from LANG)' -r -f -a "en\t''
ja\t''"
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l no-config -d 'Ignore project and global configuration files'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l no-baseline -d 'Ignore `ayame-spell-baseline.json` and report every finding'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l no-ignore -d 'Do not honour `.gitignore`, `.ignore`, or Git exclude files'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l hidden -d 'Include hidden files and directories'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -s q -l quiet -d 'Print findings only, without summaries'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -s v -l verbose -d 'Report configuration sources, skipped files, and elapsed time'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l no-cache -d 'Disable the incremental per-file scan cache'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -s w -l write -d 'Apply safe fixes in place (shorthand for `fix`)'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l list-rules -d 'List every stable issue code'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -s V -l version -d 'Print version'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "check" -d 'Check files and report issues (the default)'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "fix" -d 'Apply all safe fixes in place (single-candidate corrections and mechanical notation conversions)'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "words" -d 'Word management: bulk collection, triage, and dictionary additions'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "dict" -d 'Shared dictionaries from the ayame-spell registry'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "import" -d 'Import configuration and dictionaries from another spelling tool'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "init" -d 'Write a starter ayame-spell.toml in the current directory'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "config" -d 'Print, validate, or describe the configuration'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "baseline" -d 'Record current findings so only new findings fail later checks'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "explain" -d 'Explain a stable issue code and how to configure or silence it'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "rules" -d 'List every stable issue code'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "completions" -d 'Generate a shell completion script on standard output'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "completion-candidates" -d 'Internal, non-network completion candidate provider'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "lsp" -d 'Run the LSP server (used by editor integrations)'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l config -d 'Load exactly this configuration file' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l mode -d 'Override `[check].mode`' -r -f -a "corrections\t''
dictionary\t''
off\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l exclude -d 'Exclude an additional glob (repeatable)' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l color -d 'Colour policy for human output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l stdin-filename -d 'Display name used for standard input (also selects overrides)' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l max-file-size -d 'Skip files larger than this many bytes (overrides `[files].max-file-size`)' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -s j -l threads -d 'Worker threads (overrides the detected CPU count)' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l cache-dir -d 'Use this incremental cache directory (also enables caching in CI)' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l format -r -f -a "human\t''
brief\t''
json\t''
github\t''
sarif\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l no-config -d 'Ignore project and global configuration files'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l no-baseline -d 'Ignore `ayame-spell-baseline.json` and report every finding'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l no-ignore -d 'Do not honour `.gitignore`, `.ignore`, or Git exclude files'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l hidden -d 'Include hidden files and directories'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -s q -l quiet -d 'Print findings only, without summaries'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -s v -l verbose -d 'Report configuration sources, skipped files, and elapsed time'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l no-cache -d 'Disable the incremental per-file scan cache'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -s w -l write -d 'Apply safe fixes in place'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l config -d 'Load exactly this configuration file' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l mode -d 'Override `[check].mode`' -r -f -a "corrections\t''
dictionary\t''
off\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l exclude -d 'Exclude an additional glob (repeatable)' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l color -d 'Colour policy for human output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l stdin-filename -d 'Display name used for standard input (also selects overrides)' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l max-file-size -d 'Skip files larger than this many bytes (overrides `[files].max-file-size`)' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -s j -l threads -d 'Worker threads (overrides the detected CPU count)' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l cache-dir -d 'Use this incremental cache directory (also enables caching in CI)' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l no-config -d 'Ignore project and global configuration files'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l no-baseline -d 'Ignore `ayame-spell-baseline.json` and report every finding'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l no-ignore -d 'Do not honour `.gitignore`, `.ignore`, or Git exclude files'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l hidden -d 'Include hidden files and directories'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -s q -l quiet -d 'Print findings only, without summaries'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -s v -l verbose -d 'Report configuration sources, skipped files, and elapsed time'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l no-cache -d 'Disable the incremental per-file scan cache'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l dry-run -d 'Print a unified diff without writing files'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -l interactive -d 'Confirm or redirect each finding interactively'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and not __fish_seen_subcommand_from collect add triage help" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and not __fish_seen_subcommand_from collect add triage help" -f -a "collect" -d 'Collect flagged words across files, ranked by frequency'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and not __fish_seen_subcommand_from collect add triage help" -f -a "add" -d 'Add words to the project (default) or global word file'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and not __fish_seen_subcommand_from collect add triage help" -f -a "triage" -d 'Search flagged words and choose a dictionary, ignore, fix, or skip action for each one'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and not __fish_seen_subcommand_from collect add triage help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from collect" -l min-count -d 'Only include words flagged at least this many times' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from collect" -l plain -d 'Print bare words only (ready to append to a word file)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from collect" -l json
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from collect" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from add" -l global
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from triage" -l kind -d 'Only include one finding kind' -r -f -a "typo\t''
unknown-word\t''
en-variant\t''
ja-variant\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from triage" -l min-count -d 'Only include words flagged at least this many times' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from triage" -l limit -d 'Review at most this many words after sorting and filtering' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from triage" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from help" -f -a "collect" -d 'Collect flagged words across files, ranked by frequency'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from help" -f -a "add" -d 'Add words to the project (default) or global word file'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from help" -f -a "triage" -d 'Search flagged words and choose a dictionary, ignore, fix, or skip action for each one'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add search info remove update vendor help" -l registry -d 'Override the registry index URL' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add search info remove update vendor help" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add search info remove update vendor help" -f -a "list" -d 'List available dictionaries and their install status'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add search info remove update vendor help" -f -a "add" -d 'Download dictionaries and enable them in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add search info remove update vendor help" -f -a "search" -d 'Search registry names and descriptions'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add search info remove update vendor help" -f -a "info" -d 'Show metadata and project status for one dictionary'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add search info remove update vendor help" -f -a "remove" -d 'Delete a cached dictionary and disable it in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add search info remove update vendor help" -f -a "update" -d 'Compare cached dictionaries with the registry and update unlocked ones'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add search info remove update vendor help" -f -a "vendor" -d 'Copy a registry dictionary into the project and rewrite its config reference for offline use'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add search info remove update vendor help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from list" -l lang -d 'Filter by language' -r -f -a "en\t''
ja\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from list" -l kind -d 'Filter by dictionary kind' -r -f -a "wordlist\t''
corrections\t''
variants\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from list" -l registry -d 'Override the registry index URL' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from list" -l json -d 'Emit one JSON array for scripting'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from add" -l lang -d 'Filter the interactive picker by language' -r -f -a "en\t''
ja\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from add" -l kind -d 'Filter the interactive picker by dictionary kind' -r -f -a "wordlist\t''
corrections\t''
variants\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from add" -l registry -d 'Override the registry index URL' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from add" -l cache-only -d 'Download to the cache only; leave the project config untouched'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from search" -l lang -r -f -a "en\t''
ja\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from search" -l kind -r -f -a "wordlist\t''
corrections\t''
variants\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from search" -l registry -d 'Override the registry index URL' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from search" -l json
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from info" -l registry -d 'Override the registry index URL' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from info" -l json
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from remove" -l registry -d 'Override the registry index URL' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from update" -l registry -d 'Override the registry index URL' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from update" -l check -d 'Exit with status 1 when an update is available; write nothing'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from update" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from vendor" -l dir -d 'Project-relative destination directory' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from vendor" -l registry -d 'Override the registry index URL' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from vendor" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "list" -d 'List available dictionaries and their install status'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "add" -d 'Download dictionaries and enable them in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search registry names and descriptions'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "info" -d 'Show metadata and project status for one dictionary'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Delete a cached dictionary and disable it in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "update" -d 'Compare cached dictionaries with the registry and update unlocked ones'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "vendor" -d 'Copy a registry dictionary into the project and rewrite its config reference for offline use'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and not __fish_seen_subcommand_from cspell typos prh help" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and not __fish_seen_subcommand_from cspell typos prh help" -f -a "cspell" -d 'Import cSpell words, ignores, paths, and known dictionaries'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and not __fish_seen_subcommand_from cspell typos prh help" -f -a "typos" -d 'Import _typos.toml extend-words and extend-exclude'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and not __fish_seen_subcommand_from cspell typos prh help" -f -a "prh" -d 'Import a supported subset of prh YAML rules'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and not __fish_seen_subcommand_from cspell typos prh help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from cspell" -l dry-run -d 'Print the merged output without writing'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from cspell" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from typos" -l dry-run -d 'Print the merged output without writing'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from typos" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from prh" -l output -d 'Project-relative TOML rule file to generate' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from prh" -l dry-run -d 'Print the merged config and rule file without writing'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from prh" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from help" -f -a "cspell" -d 'Import cSpell words, ignores, paths, and known dictionaries'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from help" -f -a "typos" -d 'Import _typos.toml extend-words and extend-exclude'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from help" -f -a "prh" -d 'Import a supported subset of prh YAML rules'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand import; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand init" -l force -d 'Overwrite an existing config file'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand init" -l interactive -d 'Run the guided setup wizard'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand init" -l yes -d 'Use the non-interactive starter configuration'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand init" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand config" -l schema -d 'Print the versioned JSON Schema'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand config" -l validate -d 'Validate a configuration file (discovers the project file when PATH is omitted)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand config" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l config -d 'Load exactly this configuration file' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l mode -d 'Override `[check].mode`' -r -f -a "corrections\t''
dictionary\t''
off\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l exclude -d 'Exclude an additional glob (repeatable)' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l color -d 'Colour policy for human output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l stdin-filename -d 'Display name used for standard input (also selects overrides)' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l max-file-size -d 'Skip files larger than this many bytes (overrides `[files].max-file-size`)' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -s j -l threads -d 'Worker threads (overrides the detected CPU count)' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l cache-dir -d 'Use this incremental cache directory (also enables caching in CI)' -r -F
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l no-config -d 'Ignore project and global configuration files'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l no-baseline -d 'Ignore `ayame-spell-baseline.json` and report every finding'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l no-ignore -d 'Do not honour `.gitignore`, `.ignore`, or Git exclude files'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l hidden -d 'Include hidden files and directories'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -s q -l quiet -d 'Print findings only, without summaries'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -s v -l verbose -d 'Report configuration sources, skipped files, and elapsed time'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l no-cache -d 'Disable the incremental per-file scan cache'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -l prune -d 'Remove baseline entries whose finding no longer exists'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand baseline" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand explain" -l lang -d 'Explanation language (defaults from LANG)' -r -f -a "en\t''
ja\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand explain" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand rules" -l lang -d 'Description language (defaults from LANG)' -r -f -a "en\t''
ja\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand rules" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand completions" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand completion-candidates" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand lsp" -l stdio -d 'Use standard input/output transport. Accepted for client compatibility; stdio is always the transport'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand lsp" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "check" -d 'Check files and report issues (the default)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "fix" -d 'Apply all safe fixes in place (single-candidate corrections and mechanical notation conversions)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "words" -d 'Word management: bulk collection, triage, and dictionary additions'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "dict" -d 'Shared dictionaries from the ayame-spell registry'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "import" -d 'Import configuration and dictionaries from another spelling tool'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "init" -d 'Write a starter ayame-spell.toml in the current directory'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "config" -d 'Print, validate, or describe the configuration'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "baseline" -d 'Record current findings so only new findings fail later checks'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "explain" -d 'Explain a stable issue code and how to configure or silence it'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "rules" -d 'List every stable issue code'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "completions" -d 'Generate a shell completion script on standard output'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "completion-candidates" -d 'Internal, non-network completion candidate provider'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "lsp" -d 'Run the LSP server (used by editor integrations)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict import init config baseline explain rules completions completion-candidates lsp help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from words" -f -a "collect" -d 'Collect flagged words across files, ranked by frequency'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from words" -f -a "add" -d 'Add words to the project (default) or global word file'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from words" -f -a "triage" -d 'Search flagged words and choose a dictionary, ignore, fix, or skip action for each one'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "list" -d 'List available dictionaries and their install status'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "add" -d 'Download dictionaries and enable them in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "search" -d 'Search registry names and descriptions'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "info" -d 'Show metadata and project status for one dictionary'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "remove" -d 'Delete a cached dictionary and disable it in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "update" -d 'Compare cached dictionaries with the registry and update unlocked ones'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "vendor" -d 'Copy a registry dictionary into the project and rewrite its config reference for offline use'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from import" -f -a "cspell" -d 'Import cSpell words, ignores, paths, and known dictionaries'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from import" -f -a "typos" -d 'Import _typos.toml extend-words and extend-exclude'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from import" -f -a "prh" -d 'Import a supported subset of prh YAML rules'

# ayame-spell dynamic completion (cache-only; never performs network I/O).
complete -c ayame-spell -n '__fish_seen_subcommand_from dict; and __fish_seen_subcommand_from add' -f -a '(command ayame-spell __complete dict-add (commandline -ct))'
complete -c ayame-spell -n '__fish_seen_subcommand_from dict; and __fish_seen_subcommand_from remove' -f -a '(command ayame-spell __complete dict-remove (commandline -ct))'
complete -c ayame-spell -n '__fish_seen_subcommand_from words; and __fish_seen_subcommand_from add' -f -a '(command ayame-spell __complete words-add (commandline -ct))'
