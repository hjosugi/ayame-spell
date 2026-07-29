# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_ayame_spell_global_optspecs
	string join \n w/write format= j/threads= h/help V/version
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

complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -l format -d 'Output format' -r -f -a "human\t''
brief\t''
json\t''"
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -s j -l threads -d 'Worker threads (default: number of CPUs)' -r
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -s w -l write -d 'Apply safe fixes in place (shorthand for `fix`)'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -s V -l version -d 'Print version'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "check" -d 'Check files and report issues (the default)'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "fix" -d 'Apply all safe fixes in place (single-candidate corrections and mechanical notation conversions)'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "words" -d 'Word management: bulk collection, triage, and dictionary additions'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "dict" -d 'Shared dictionaries from the ayame-spell registry'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "init" -d 'Write a starter ayame-spell.toml in the current directory'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "config" -d 'Print the effective merged configuration'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "completions" -d 'Generate a shell completion script on standard output'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "lsp" -d 'Run the LSP server (used by editor integrations)'
complete -c ayame-spell -n "__fish_ayame_spell_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -l format -r -f -a "human\t''
brief\t''
json\t''"
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -s j -l threads -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -s w -l write -d 'Apply safe fixes in place'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand check" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -s j -l threads -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand fix" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and not __fish_seen_subcommand_from collect add triage help" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and not __fish_seen_subcommand_from collect add triage help" -f -a "collect" -d 'Collect flagged words across files, ranked by frequency'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and not __fish_seen_subcommand_from collect add triage help" -f -a "add" -d 'Add words to the project (default) or global word file'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and not __fish_seen_subcommand_from collect add triage help" -f -a "triage" -d 'Interactive bulk triage of flagged words: multi-select what goes to the project dictionary, the global dictionary, or the ignore list'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and not __fish_seen_subcommand_from collect add triage help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from collect" -l min-count -d 'Only include words flagged at least this many times' -r
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from collect" -l plain -d 'Print bare words only (ready to append to a word file)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from collect" -l json
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from collect" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from add" -l global
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from triage" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from help" -f -a "collect" -d 'Collect flagged words across files, ranked by frequency'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from help" -f -a "add" -d 'Add words to the project (default) or global word file'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from help" -f -a "triage" -d 'Interactive bulk triage of flagged words: multi-select what goes to the project dictionary, the global dictionary, or the ignore list'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand words; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add remove update help" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add remove update help" -f -a "list" -d 'List available dictionaries and their install status'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add remove update help" -f -a "add" -d 'Download dictionaries and enable them in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add remove update help" -f -a "remove" -d 'Delete a cached dictionary and disable it in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add remove update help" -f -a "update" -d 'Re-download every cached dictionary from the registry'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and not __fish_seen_subcommand_from list add remove update help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from add" -l cache-only -d 'Download to the cache only; leave the project config untouched'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from update" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "list" -d 'List available dictionaries and their install status'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "add" -d 'Download dictionaries and enable them in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Delete a cached dictionary and disable it in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "update" -d 'Re-download every cached dictionary from the registry'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand dict; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand init" -l force -d 'Overwrite an existing config file'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand init" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand config" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand completions" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand lsp" -l stdio -d 'Use standard input/output transport. Accepted for client compatibility; stdio is always the transport'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand lsp" -s h -l help -d 'Print help'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict init config completions lsp help" -f -a "check" -d 'Check files and report issues (the default)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict init config completions lsp help" -f -a "fix" -d 'Apply all safe fixes in place (single-candidate corrections and mechanical notation conversions)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict init config completions lsp help" -f -a "words" -d 'Word management: bulk collection, triage, and dictionary additions'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict init config completions lsp help" -f -a "dict" -d 'Shared dictionaries from the ayame-spell registry'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict init config completions lsp help" -f -a "init" -d 'Write a starter ayame-spell.toml in the current directory'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict init config completions lsp help" -f -a "config" -d 'Print the effective merged configuration'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict init config completions lsp help" -f -a "completions" -d 'Generate a shell completion script on standard output'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict init config completions lsp help" -f -a "lsp" -d 'Run the LSP server (used by editor integrations)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and not __fish_seen_subcommand_from check fix words dict init config completions lsp help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from words" -f -a "collect" -d 'Collect flagged words across files, ranked by frequency'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from words" -f -a "add" -d 'Add words to the project (default) or global word file'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from words" -f -a "triage" -d 'Interactive bulk triage of flagged words: multi-select what goes to the project dictionary, the global dictionary, or the ignore list'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "list" -d 'List available dictionaries and their install status'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "add" -d 'Download dictionaries and enable them in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "remove" -d 'Delete a cached dictionary and disable it in the project config'
complete -c ayame-spell -n "__fish_ayame_spell_using_subcommand help; and __fish_seen_subcommand_from dict" -f -a "update" -d 'Re-download every cached dictionary from the registry'
