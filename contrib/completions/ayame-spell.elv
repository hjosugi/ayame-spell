
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
            cand --format 'Output format'
            cand -j 'Worker threads (default: number of CPUs)'
            cand --threads 'Worker threads (default: number of CPUs)'
            cand -w 'Apply safe fixes in place (shorthand for `fix`)'
            cand --write 'Apply safe fixes in place (shorthand for `fix`)'
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
            cand completions 'Generate a shell completion script on standard output'
            cand lsp 'Run the LSP server (used by editor integrations)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'ayame-spell;check'= {
            cand --format 'format'
            cand -j 'j'
            cand --threads 'threads'
            cand -w 'Apply safe fixes in place'
            cand --write 'Apply safe fixes in place'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;fix'= {
            cand -j 'j'
            cand --threads 'threads'
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
            cand remove 'Delete a cached dictionary and disable it in the project config'
            cand update 'Re-download every cached dictionary from the registry'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'ayame-spell;dict;list'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;dict;add'= {
            cand --cache-only 'Download to the cache only; leave the project config untouched'
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
            cand remove 'Delete a cached dictionary and disable it in the project config'
            cand update 'Re-download every cached dictionary from the registry'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'ayame-spell;dict;help;list'= {
        }
        &'ayame-spell;dict;help;add'= {
        }
        &'ayame-spell;dict;help;remove'= {
        }
        &'ayame-spell;dict;help;update'= {
        }
        &'ayame-spell;dict;help;help'= {
        }
        &'ayame-spell;init'= {
            cand --force 'Overwrite an existing config file'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;config'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'ayame-spell;completions'= {
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
            cand completions 'Generate a shell completion script on standard output'
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
            cand remove 'Delete a cached dictionary and disable it in the project config'
            cand update 'Re-download every cached dictionary from the registry'
        }
        &'ayame-spell;help;dict;list'= {
        }
        &'ayame-spell;help;dict;add'= {
        }
        &'ayame-spell;help;dict;remove'= {
        }
        &'ayame-spell;help;dict;update'= {
        }
        &'ayame-spell;help;init'= {
        }
        &'ayame-spell;help;config'= {
        }
        &'ayame-spell;help;completions'= {
        }
        &'ayame-spell;help;lsp'= {
        }
        &'ayame-spell;help;help'= {
        }
    ]
    if (eq $words[-2] --format) {
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
