
use builtin;
use str;

set edit:completion:arg-completer[ayame-spell] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'ayame-spell'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'ayame-spell'= {
            cand --config 'Load exactly this configuration file'
            cand --mode 'Override `[check].mode`'
            cand --exclude 'Exclude an additional glob (repeatable)'
            cand --color 'Colour policy for human output'
            cand --stdin-filename 'Display name used for standard input (also selects overrides)'
            cand --max-file-size 'Skip files larger than this many bytes (overrides `[files].max-file-size`)'
            cand -j 'Worker threads (overrides the detected CPU count)'
            cand --threads 'Worker threads (overrides the detected CPU count)'
            cand --format 'Output format'
            cand --lang 'Language for `--list-rules` (defaults from LANG)'
            cand --no-config 'Ignore project and global configuration files'
            cand --no-ignore 'Do not honour `.gitignore`, `.ignore`, or Git exclude files'
            cand --hidden 'Include hidden files and directories'
            cand -q 'Print findings only, without summaries'
            cand --quiet 'Print findings only, without summaries'
            cand -v 'Report configuration sources, skipped files, and elapsed time'
            cand --verbose 'Report configuration sources, skipped files, and elapsed time'
            cand -w 'Apply safe fixes in place (shorthand for `fix`)'
            cand --write 'Apply safe fixes in place (shorthand for `fix`)'
            cand --list-rules 'List every stable issue code'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
            cand check 'Check files and report issues (the default)'
            cand fix 'Apply all safe fixes in place (single-candidate corrections and mechanical notation conversions)'
            cand words 'Word management: bulk collection, triage, and dictionary additions'
            cand dict 'Shared dictionaries from the ayame-spell registry'
            cand init 'Write a starter ayame-spell.toml in the current directory'
            cand config 'Print the effective merged configuration'
            cand explain 'Explain a stable issue code and how to configure or silence it'
            cand rules 'List every stable issue code'
            cand completions 'Generate a shell completion script on standard output'
            cand completion-candidates 'Internal, non-network completion candidate provider'
            cand lsp 'Run the LSP server (used by editor integrations)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'ayame-spell;check'= {
            cand --config 'Load exactly this configuration file'
            cand --mode 'Override `[check].mode`'
            cand --exclude 'Exclude an additional glob (repeatable)'
            cand --color 'Colour policy for human output'
            cand --stdin-filename 'Display name used for standard input (also selects overrides)'
            cand --max-file-size 'Skip files larger than this many bytes (overrides `[files].max-file-size`)'
            cand -j 'Worker threads (overrides the detected CPU count)'
            cand --threads 'Worker threads (overrides the detected CPU count)'
            cand --format 'format'
            cand --no-config 'Ignore project and global configuration files'
            cand --no-ignore 'Do not honour `.gitignore`, `.ignore`, or Git exclude files'
            cand --hidden 'Include hidden files and directories'
            cand -q 'Print findings only, without summaries'
            cand --quiet 'Print findings only, without summaries'
            cand -v 'Report configuration sources, skipped files, and elapsed time'
            cand --verbose 'Report configuration sources, skipped files, and elapsed time'
            cand -w 'Apply safe fixes in place'
            cand --write 'Apply safe fixes in place'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;fix'= {
            cand --config 'Load exactly this configuration file'
            cand --mode 'Override `[check].mode`'
            cand --exclude 'Exclude an additional glob (repeatable)'
            cand --color 'Colour policy for human output'
            cand --stdin-filename 'Display name used for standard input (also selects overrides)'
            cand --max-file-size 'Skip files larger than this many bytes (overrides `[files].max-file-size`)'
            cand -j 'Worker threads (overrides the detected CPU count)'
            cand --threads 'Worker threads (overrides the detected CPU count)'
            cand --no-config 'Ignore project and global configuration files'
            cand --no-ignore 'Do not honour `.gitignore`, `.ignore`, or Git exclude files'
            cand --hidden 'Include hidden files and directories'
            cand -q 'Print findings only, without summaries'
            cand --quiet 'Print findings only, without summaries'
            cand -v 'Report configuration sources, skipped files, and elapsed time'
            cand --verbose 'Report configuration sources, skipped files, and elapsed time'
            cand --dry-run 'Print a unified diff without writing files'
            cand --interactive 'Confirm or redirect each finding interactively'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;words'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand collect 'Collect flagged words across files, ranked by frequency'
            cand add 'Add words to the project (default) or global word file'
            cand triage 'Interactive bulk triage of flagged words: multi-select what goes to the project dictionary, the global dictionary, or the ignore list'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'ayame-spell;words;collect'= {
            cand --min-count 'Only include words flagged at least this many times'
            cand --plain 'Print bare words only (ready to append to a word file)'
            cand --json 'json'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;words;add'= {
            cand --global 'global'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;words;triage'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;words;help'= {
            cand collect 'Collect flagged words across files, ranked by frequency'
            cand add 'Add words to the project (default) or global word file'
            cand triage 'Interactive bulk triage of flagged words: multi-select what goes to the project dictionary, the global dictionary, or the ignore list'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'ayame-spell;words;help;collect'= {
        }
        &'ayame-spell;words;help;add'= {
        }
        &'ayame-spell;words;help;triage'= {
        }
        &'ayame-spell;words;help;help'= {
        }
        &'ayame-spell;dict'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand list 'List available dictionaries and their install status'
            cand add 'Download dictionaries and enable them in the project config'
            cand search 'Search registry names and descriptions'
            cand info 'Show metadata and project status for one dictionary'
            cand remove 'Delete a cached dictionary and disable it in the project config'
            cand update 'Re-download every cached dictionary from the registry'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'ayame-spell;dict;list'= {
            cand --lang 'Filter by language'
            cand --kind 'Filter by dictionary kind'
            cand --json 'Emit one JSON array for scripting'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;dict;add'= {
            cand --lang 'Filter the interactive picker by language'
            cand --kind 'Filter the interactive picker by dictionary kind'
            cand --cache-only 'Download to the cache only; leave the project config untouched'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;dict;search'= {
            cand --lang 'lang'
            cand --kind 'kind'
            cand --json 'json'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;dict;info'= {
            cand --json 'json'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;dict;remove'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;dict;update'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;dict;help'= {
            cand list 'List available dictionaries and their install status'
            cand add 'Download dictionaries and enable them in the project config'
            cand search 'Search registry names and descriptions'
            cand info 'Show metadata and project status for one dictionary'
            cand remove 'Delete a cached dictionary and disable it in the project config'
            cand update 'Re-download every cached dictionary from the registry'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'ayame-spell;dict;help;list'= {
        }
        &'ayame-spell;dict;help;add'= {
        }
        &'ayame-spell;dict;help;search'= {
        }
        &'ayame-spell;dict;help;info'= {
        }
        &'ayame-spell;dict;help;remove'= {
        }
        &'ayame-spell;dict;help;update'= {
        }
        &'ayame-spell;dict;help;help'= {
        }
        &'ayame-spell;init'= {
            cand --force 'Overwrite an existing config file'
            cand --interactive 'Run the guided setup wizard'
            cand --yes 'Use the non-interactive starter configuration'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;config'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;explain'= {
            cand --lang 'Explanation language (defaults from LANG)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;rules'= {
            cand --lang 'Description language (defaults from LANG)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;completions'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;completion-candidates'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;lsp'= {
            cand --stdio 'Use standard input/output transport. Accepted for client compatibility; stdio is always the transport'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;help'= {
            cand check 'Check files and report issues (the default)'
            cand fix 'Apply all safe fixes in place (single-candidate corrections and mechanical notation conversions)'
            cand words 'Word management: bulk collection, triage, and dictionary additions'
            cand dict 'Shared dictionaries from the ayame-spell registry'
            cand init 'Write a starter ayame-spell.toml in the current directory'
            cand config 'Print the effective merged configuration'
            cand explain 'Explain a stable issue code and how to configure or silence it'
            cand rules 'List every stable issue code'
            cand completions 'Generate a shell completion script on standard output'
            cand completion-candidates 'Internal, non-network completion candidate provider'
            cand lsp 'Run the LSP server (used by editor integrations)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'ayame-spell;help;check'= {
        }
        &'ayame-spell;help;fix'= {
        }
        &'ayame-spell;help;words'= {
            cand collect 'Collect flagged words across files, ranked by frequency'
            cand add 'Add words to the project (default) or global word file'
            cand triage 'Interactive bulk triage of flagged words: multi-select what goes to the project dictionary, the global dictionary, or the ignore list'
        }
        &'ayame-spell;help;words;collect'= {
        }
        &'ayame-spell;help;words;add'= {
        }
        &'ayame-spell;help;words;triage'= {
        }
        &'ayame-spell;help;dict'= {
            cand list 'List available dictionaries and their install status'
            cand add 'Download dictionaries and enable them in the project config'
            cand search 'Search registry names and descriptions'
            cand info 'Show metadata and project status for one dictionary'
            cand remove 'Delete a cached dictionary and disable it in the project config'
            cand update 'Re-download every cached dictionary from the registry'
        }
        &'ayame-spell;help;dict;list'= {
        }
        &'ayame-spell;help;dict;add'= {
        }
        &'ayame-spell;help;dict;search'= {
        }
        &'ayame-spell;help;dict;info'= {
        }
        &'ayame-spell;help;dict;remove'= {
        }
        &'ayame-spell;help;dict;update'= {
        }
        &'ayame-spell;help;init'= {
        }
        &'ayame-spell;help;config'= {
        }
        &'ayame-spell;help;explain'= {
        }
        &'ayame-spell;help;rules'= {
        }
        &'ayame-spell;help;completions'= {
        }
        &'ayame-spell;help;completion-candidates'= {
        }
        &'ayame-spell;help;lsp'= {
        }
        &'ayame-spell;help;help'= {
        }
    ]
    if (and (>= (count $words) 3) (eq $words[-3] dict) (eq $words[-2] add)) {
        each {|candidate| cand $candidate 'Registry dictionary'} (ayame-spell __complete dict-add $words[-1] | from-lines)
    } elif (and (>= (count $words) 3) (eq $words[-3] dict) (eq $words[-2] remove)) {
        each {|candidate| cand $candidate 'Installed dictionary'} (ayame-spell __complete dict-remove $words[-1] | from-lines)
    } elif (and (>= (count $words) 3) (eq $words[-3] words) (eq $words[-2] add)) {
        each {|candidate| cand $candidate 'Flagged word'} (ayame-spell __complete words-add $words[-1] | from-lines)
    } elif (eq $words[-2] --format) {
        cand human 'Output format'
        cand brief 'Output format'
        cand json 'Output format'
    } elif (eq $words[-2] completions) {
        cand bash 'Shell'
        cand elvish 'Shell'
        cand fish 'Shell'
        cand powershell 'Shell'
        cand zsh 'Shell'
    } else {
        $completions[$command]
    }
}
