
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'ayame-spell' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $lastElement = $commandElements[$commandElements.Count - 1]
    $valueFor = if ($lastElement.Value -in @('--format', 'completions')) {
        $lastElement.Value
    } elseif ($commandElements.Count -ge 3 -and
              $lastElement.Value -eq $wordToComplete) {
        $commandElements[$commandElements.Count - 2].Value
    }

    $values = switch ($valueFor) {
        '--format' { @('human', 'brief', 'json') }
        'completions' { @('bash', 'elvish', 'fish', 'powershell', 'zsh') }
    }
    if ($null -ne $values) {
        $values |
            Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object {
                [CompletionResult]::new(
                    $_, $_, [CompletionResultType]::ParameterValue, $_)
            }
        return
    }

    $command = @(
        'ayame-spell'
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
        'ayame-spell' {
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'Worker threads (default: number of CPUs)')
            [CompletionResult]::new('--threads', '--threads', [CompletionResultType]::ParameterName, 'Worker threads (default: number of CPUs)')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Apply safe fixes in place (shorthand for `fix`)')
            [CompletionResult]::new('--write', '--write', [CompletionResultType]::ParameterName, 'Apply safe fixes in place (shorthand for `fix`)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('check', 'check', [CompletionResultType]::ParameterValue, 'Check files and report issues (the default)')
            [CompletionResult]::new('fix', 'fix', [CompletionResultType]::ParameterValue, 'Apply all safe fixes in place (single-candidate corrections and mechanical notation conversions)')
            [CompletionResult]::new('words', 'words', [CompletionResultType]::ParameterValue, 'Word management: bulk collection, triage, and dictionary additions')
            [CompletionResult]::new('dict', 'dict', [CompletionResultType]::ParameterValue, 'Shared dictionaries from the ayame-spell registry')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Write a starter ayame-spell.toml in the current directory')
            [CompletionResult]::new('config', 'config', [CompletionResultType]::ParameterValue, 'Print the effective merged configuration')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate a shell completion script on standard output')
            [CompletionResult]::new('lsp', 'lsp', [CompletionResultType]::ParameterValue, 'Run the LSP server (used by editor integrations)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'ayame-spell;check' {
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'format')
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'j')
            [CompletionResult]::new('--threads', '--threads', [CompletionResultType]::ParameterName, 'threads')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Apply safe fixes in place')
            [CompletionResult]::new('--write', '--write', [CompletionResultType]::ParameterName, 'Apply safe fixes in place')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;fix' {
            [CompletionResult]::new('-j', '-j', [CompletionResultType]::ParameterName, 'j')
            [CompletionResult]::new('--threads', '--threads', [CompletionResultType]::ParameterName, 'threads')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;words' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('collect', 'collect', [CompletionResultType]::ParameterValue, 'Collect flagged words across files, ranked by frequency')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add words to the project (default) or global word file')
            [CompletionResult]::new('triage', 'triage', [CompletionResultType]::ParameterValue, 'Interactive bulk triage of flagged words: multi-select what goes to the project dictionary, the global dictionary, or the ignore list')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'ayame-spell;words;collect' {
            [CompletionResult]::new('--min-count', '--min-count', [CompletionResultType]::ParameterName, 'Only include words flagged at least this many times')
            [CompletionResult]::new('--plain', '--plain', [CompletionResultType]::ParameterName, 'Print bare words only (ready to append to a word file)')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;words;add' {
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'global')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;words;triage' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;words;help' {
            [CompletionResult]::new('collect', 'collect', [CompletionResultType]::ParameterValue, 'Collect flagged words across files, ranked by frequency')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add words to the project (default) or global word file')
            [CompletionResult]::new('triage', 'triage', [CompletionResultType]::ParameterValue, 'Interactive bulk triage of flagged words: multi-select what goes to the project dictionary, the global dictionary, or the ignore list')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'ayame-spell;words;help;collect' {
            break
        }
        'ayame-spell;words;help;add' {
            break
        }
        'ayame-spell;words;help;triage' {
            break
        }
        'ayame-spell;words;help;help' {
            break
        }
        'ayame-spell;dict' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List available dictionaries and their install status')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Download dictionaries and enable them in the project config')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Delete a cached dictionary and disable it in the project config')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Re-download every cached dictionary from the registry')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'ayame-spell;dict;list' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;dict;add' {
            [CompletionResult]::new('--cache-only', '--cache-only', [CompletionResultType]::ParameterName, 'Download to the cache only; leave the project config untouched')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;dict;remove' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;dict;update' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;dict;help' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List available dictionaries and their install status')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Download dictionaries and enable them in the project config')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Delete a cached dictionary and disable it in the project config')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Re-download every cached dictionary from the registry')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'ayame-spell;dict;help;list' {
            break
        }
        'ayame-spell;dict;help;add' {
            break
        }
        'ayame-spell;dict;help;remove' {
            break
        }
        'ayame-spell;dict;help;update' {
            break
        }
        'ayame-spell;dict;help;help' {
            break
        }
        'ayame-spell;init' {
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Overwrite an existing config file')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;config' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;completions' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;lsp' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'ayame-spell;help' {
            [CompletionResult]::new('check', 'check', [CompletionResultType]::ParameterValue, 'Check files and report issues (the default)')
            [CompletionResult]::new('fix', 'fix', [CompletionResultType]::ParameterValue, 'Apply all safe fixes in place (single-candidate corrections and mechanical notation conversions)')
            [CompletionResult]::new('words', 'words', [CompletionResultType]::ParameterValue, 'Word management: bulk collection, triage, and dictionary additions')
            [CompletionResult]::new('dict', 'dict', [CompletionResultType]::ParameterValue, 'Shared dictionaries from the ayame-spell registry')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Write a starter ayame-spell.toml in the current directory')
            [CompletionResult]::new('config', 'config', [CompletionResultType]::ParameterValue, 'Print the effective merged configuration')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate a shell completion script on standard output')
            [CompletionResult]::new('lsp', 'lsp', [CompletionResultType]::ParameterValue, 'Run the LSP server (used by editor integrations)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'ayame-spell;help;check' {
            break
        }
        'ayame-spell;help;fix' {
            break
        }
        'ayame-spell;help;words' {
            [CompletionResult]::new('collect', 'collect', [CompletionResultType]::ParameterValue, 'Collect flagged words across files, ranked by frequency')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add words to the project (default) or global word file')
            [CompletionResult]::new('triage', 'triage', [CompletionResultType]::ParameterValue, 'Interactive bulk triage of flagged words: multi-select what goes to the project dictionary, the global dictionary, or the ignore list')
            break
        }
        'ayame-spell;help;words;collect' {
            break
        }
        'ayame-spell;help;words;add' {
            break
        }
        'ayame-spell;help;words;triage' {
            break
        }
        'ayame-spell;help;dict' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List available dictionaries and their install status')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Download dictionaries and enable them in the project config')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Delete a cached dictionary and disable it in the project config')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Re-download every cached dictionary from the registry')
            break
        }
        'ayame-spell;help;dict;list' {
            break
        }
        'ayame-spell;help;dict;add' {
            break
        }
        'ayame-spell;help;dict;remove' {
            break
        }
        'ayame-spell;help;dict;update' {
            break
        }
        'ayame-spell;help;init' {
            break
        }
        'ayame-spell;help;config' {
            break
        }
        'ayame-spell;help;completions' {
            break
        }
        'ayame-spell;help;lsp' {
            break
        }
        'ayame-spell;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
