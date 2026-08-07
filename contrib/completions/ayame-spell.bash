_ayame-spell() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="ayame__spell"
                ;;
            ayame__spell,baseline)
                cmd="ayame__spell__subcmd__baseline"
                ;;
            ayame__spell,check)
                cmd="ayame__spell__subcmd__check"
                ;;
            ayame__spell,completion-candidates)
                cmd="ayame__spell__subcmd__completion__subcmd__candidates"
                ;;
            ayame__spell,completions)
                cmd="ayame__spell__subcmd__completions"
                ;;
            ayame__spell,config)
                cmd="ayame__spell__subcmd__config"
                ;;
            ayame__spell,dict)
                cmd="ayame__spell__subcmd__dict"
                ;;
            ayame__spell,explain)
                cmd="ayame__spell__subcmd__explain"
                ;;
            ayame__spell,fix)
                cmd="ayame__spell__subcmd__fix"
                ;;
            ayame__spell,help)
                cmd="ayame__spell__subcmd__help"
                ;;
            ayame__spell,import)
                cmd="ayame__spell__subcmd__import"
                ;;
            ayame__spell,init)
                cmd="ayame__spell__subcmd__init"
                ;;
            ayame__spell,lsp)
                cmd="ayame__spell__subcmd__lsp"
                ;;
            ayame__spell,rules)
                cmd="ayame__spell__subcmd__rules"
                ;;
            ayame__spell,words)
                cmd="ayame__spell__subcmd__words"
                ;;
            ayame__spell__subcmd__dict,add)
                cmd="ayame__spell__subcmd__dict__subcmd__add"
                ;;
            ayame__spell__subcmd__dict,help)
                cmd="ayame__spell__subcmd__dict__subcmd__help"
                ;;
            ayame__spell__subcmd__dict,info)
                cmd="ayame__spell__subcmd__dict__subcmd__info"
                ;;
            ayame__spell__subcmd__dict,list)
                cmd="ayame__spell__subcmd__dict__subcmd__list"
                ;;
            ayame__spell__subcmd__dict,remove)
                cmd="ayame__spell__subcmd__dict__subcmd__remove"
                ;;
            ayame__spell__subcmd__dict,search)
                cmd="ayame__spell__subcmd__dict__subcmd__search"
                ;;
            ayame__spell__subcmd__dict,update)
                cmd="ayame__spell__subcmd__dict__subcmd__update"
                ;;
            ayame__spell__subcmd__dict,vendor)
                cmd="ayame__spell__subcmd__dict__subcmd__vendor"
                ;;
            ayame__spell__subcmd__dict__subcmd__help,add)
                cmd="ayame__spell__subcmd__dict__subcmd__help__subcmd__add"
                ;;
            ayame__spell__subcmd__dict__subcmd__help,help)
                cmd="ayame__spell__subcmd__dict__subcmd__help__subcmd__help"
                ;;
            ayame__spell__subcmd__dict__subcmd__help,info)
                cmd="ayame__spell__subcmd__dict__subcmd__help__subcmd__info"
                ;;
            ayame__spell__subcmd__dict__subcmd__help,list)
                cmd="ayame__spell__subcmd__dict__subcmd__help__subcmd__list"
                ;;
            ayame__spell__subcmd__dict__subcmd__help,remove)
                cmd="ayame__spell__subcmd__dict__subcmd__help__subcmd__remove"
                ;;
            ayame__spell__subcmd__dict__subcmd__help,search)
                cmd="ayame__spell__subcmd__dict__subcmd__help__subcmd__search"
                ;;
            ayame__spell__subcmd__dict__subcmd__help,update)
                cmd="ayame__spell__subcmd__dict__subcmd__help__subcmd__update"
                ;;
            ayame__spell__subcmd__dict__subcmd__help,vendor)
                cmd="ayame__spell__subcmd__dict__subcmd__help__subcmd__vendor"
                ;;
            ayame__spell__subcmd__help,baseline)
                cmd="ayame__spell__subcmd__help__subcmd__baseline"
                ;;
            ayame__spell__subcmd__help,check)
                cmd="ayame__spell__subcmd__help__subcmd__check"
                ;;
            ayame__spell__subcmd__help,completion-candidates)
                cmd="ayame__spell__subcmd__help__subcmd__completion__subcmd__candidates"
                ;;
            ayame__spell__subcmd__help,completions)
                cmd="ayame__spell__subcmd__help__subcmd__completions"
                ;;
            ayame__spell__subcmd__help,config)
                cmd="ayame__spell__subcmd__help__subcmd__config"
                ;;
            ayame__spell__subcmd__help,dict)
                cmd="ayame__spell__subcmd__help__subcmd__dict"
                ;;
            ayame__spell__subcmd__help,explain)
                cmd="ayame__spell__subcmd__help__subcmd__explain"
                ;;
            ayame__spell__subcmd__help,fix)
                cmd="ayame__spell__subcmd__help__subcmd__fix"
                ;;
            ayame__spell__subcmd__help,help)
                cmd="ayame__spell__subcmd__help__subcmd__help"
                ;;
            ayame__spell__subcmd__help,import)
                cmd="ayame__spell__subcmd__help__subcmd__import"
                ;;
            ayame__spell__subcmd__help,init)
                cmd="ayame__spell__subcmd__help__subcmd__init"
                ;;
            ayame__spell__subcmd__help,lsp)
                cmd="ayame__spell__subcmd__help__subcmd__lsp"
                ;;
            ayame__spell__subcmd__help,rules)
                cmd="ayame__spell__subcmd__help__subcmd__rules"
                ;;
            ayame__spell__subcmd__help,words)
                cmd="ayame__spell__subcmd__help__subcmd__words"
                ;;
            ayame__spell__subcmd__help__subcmd__dict,add)
                cmd="ayame__spell__subcmd__help__subcmd__dict__subcmd__add"
                ;;
            ayame__spell__subcmd__help__subcmd__dict,info)
                cmd="ayame__spell__subcmd__help__subcmd__dict__subcmd__info"
                ;;
            ayame__spell__subcmd__help__subcmd__dict,list)
                cmd="ayame__spell__subcmd__help__subcmd__dict__subcmd__list"
                ;;
            ayame__spell__subcmd__help__subcmd__dict,remove)
                cmd="ayame__spell__subcmd__help__subcmd__dict__subcmd__remove"
                ;;
            ayame__spell__subcmd__help__subcmd__dict,search)
                cmd="ayame__spell__subcmd__help__subcmd__dict__subcmd__search"
                ;;
            ayame__spell__subcmd__help__subcmd__dict,update)
                cmd="ayame__spell__subcmd__help__subcmd__dict__subcmd__update"
                ;;
            ayame__spell__subcmd__help__subcmd__dict,vendor)
                cmd="ayame__spell__subcmd__help__subcmd__dict__subcmd__vendor"
                ;;
            ayame__spell__subcmd__help__subcmd__import,cspell)
                cmd="ayame__spell__subcmd__help__subcmd__import__subcmd__cspell"
                ;;
            ayame__spell__subcmd__help__subcmd__import,prh)
                cmd="ayame__spell__subcmd__help__subcmd__import__subcmd__prh"
                ;;
            ayame__spell__subcmd__help__subcmd__import,typos)
                cmd="ayame__spell__subcmd__help__subcmd__import__subcmd__typos"
                ;;
            ayame__spell__subcmd__help__subcmd__words,add)
                cmd="ayame__spell__subcmd__help__subcmd__words__subcmd__add"
                ;;
            ayame__spell__subcmd__help__subcmd__words,collect)
                cmd="ayame__spell__subcmd__help__subcmd__words__subcmd__collect"
                ;;
            ayame__spell__subcmd__help__subcmd__words,triage)
                cmd="ayame__spell__subcmd__help__subcmd__words__subcmd__triage"
                ;;
            ayame__spell__subcmd__import,cspell)
                cmd="ayame__spell__subcmd__import__subcmd__cspell"
                ;;
            ayame__spell__subcmd__import,help)
                cmd="ayame__spell__subcmd__import__subcmd__help"
                ;;
            ayame__spell__subcmd__import,prh)
                cmd="ayame__spell__subcmd__import__subcmd__prh"
                ;;
            ayame__spell__subcmd__import,typos)
                cmd="ayame__spell__subcmd__import__subcmd__typos"
                ;;
            ayame__spell__subcmd__import__subcmd__help,cspell)
                cmd="ayame__spell__subcmd__import__subcmd__help__subcmd__cspell"
                ;;
            ayame__spell__subcmd__import__subcmd__help,help)
                cmd="ayame__spell__subcmd__import__subcmd__help__subcmd__help"
                ;;
            ayame__spell__subcmd__import__subcmd__help,prh)
                cmd="ayame__spell__subcmd__import__subcmd__help__subcmd__prh"
                ;;
            ayame__spell__subcmd__import__subcmd__help,typos)
                cmd="ayame__spell__subcmd__import__subcmd__help__subcmd__typos"
                ;;
            ayame__spell__subcmd__words,add)
                cmd="ayame__spell__subcmd__words__subcmd__add"
                ;;
            ayame__spell__subcmd__words,collect)
                cmd="ayame__spell__subcmd__words__subcmd__collect"
                ;;
            ayame__spell__subcmd__words,help)
                cmd="ayame__spell__subcmd__words__subcmd__help"
                ;;
            ayame__spell__subcmd__words,triage)
                cmd="ayame__spell__subcmd__words__subcmd__triage"
                ;;
            ayame__spell__subcmd__words__subcmd__help,add)
                cmd="ayame__spell__subcmd__words__subcmd__help__subcmd__add"
                ;;
            ayame__spell__subcmd__words__subcmd__help,collect)
                cmd="ayame__spell__subcmd__words__subcmd__help__subcmd__collect"
                ;;
            ayame__spell__subcmd__words__subcmd__help,help)
                cmd="ayame__spell__subcmd__words__subcmd__help__subcmd__help"
                ;;
            ayame__spell__subcmd__words__subcmd__help,triage)
                cmd="ayame__spell__subcmd__words__subcmd__help__subcmd__triage"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        ayame__spell)
            opts="-q -v -j -w -h -V --config --no-config --no-baseline --mode --exclude --no-ignore --hidden --color --quiet --verbose --stdin-filename --max-file-size --threads --no-cache --cache-dir --write --format --list-rules --lang --help --version check fix words dict import init config baseline explain rules completions completion-candidates lsp help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mode)
                    COMPREPLY=($(compgen -W "corrections dictionary off" -- "${cur}"))
                    return 0
                    ;;
                --exclude)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --stdin-filename)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-file-size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --threads)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -j)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cache-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --format)
                    COMPREPLY=($(compgen -W "human brief json github sarif" -- "${cur}"))
                    return 0
                    ;;
                --lang)
                    COMPREPLY=($(compgen -W "en ja" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__baseline)
            opts="-q -v -j -h --config --no-config --no-baseline --mode --exclude --no-ignore --hidden --color --quiet --verbose --stdin-filename --max-file-size --threads --no-cache --cache-dir --prune --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mode)
                    COMPREPLY=($(compgen -W "corrections dictionary off" -- "${cur}"))
                    return 0
                    ;;
                --exclude)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --stdin-filename)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-file-size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --threads)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -j)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cache-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__check)
            opts="-q -v -j -w -h --config --no-config --no-baseline --mode --exclude --no-ignore --hidden --color --quiet --verbose --stdin-filename --max-file-size --threads --no-cache --cache-dir --write --format --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mode)
                    COMPREPLY=($(compgen -W "corrections dictionary off" -- "${cur}"))
                    return 0
                    ;;
                --exclude)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --stdin-filename)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-file-size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --threads)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -j)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cache-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --format)
                    COMPREPLY=($(compgen -W "human brief json github sarif" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__completion__subcmd__candidates)
            opts="-h --help dict-add dict-remove words-add word-file config-key"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__completions)
            opts="-h --help bash elvish fish powershell zsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__config)
            opts="-h --schema --validate --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict)
            opts="-h --registry --help list add search info remove update vendor help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --registry)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__add)
            opts="-h --cache-only --lang --kind --registry --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --lang)
                    COMPREPLY=($(compgen -W "en ja" -- "${cur}"))
                    return 0
                    ;;
                --kind)
                    COMPREPLY=($(compgen -W "wordlist corrections variants" -- "${cur}"))
                    return 0
                    ;;
                --registry)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__help)
            opts="list add search info remove update vendor help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__help__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__help__subcmd__info)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__help__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__help__subcmd__search)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__help__subcmd__update)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__help__subcmd__vendor)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__info)
            opts="-h --json --registry --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --registry)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__list)
            opts="-h --json --lang --kind --registry --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --lang)
                    COMPREPLY=($(compgen -W "en ja" -- "${cur}"))
                    return 0
                    ;;
                --kind)
                    COMPREPLY=($(compgen -W "wordlist corrections variants" -- "${cur}"))
                    return 0
                    ;;
                --registry)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__remove)
            opts="-h --registry --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --registry)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__search)
            opts="-h --lang --kind --json --registry --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --lang)
                    COMPREPLY=($(compgen -W "en ja" -- "${cur}"))
                    return 0
                    ;;
                --kind)
                    COMPREPLY=($(compgen -W "wordlist corrections variants" -- "${cur}"))
                    return 0
                    ;;
                --registry)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__update)
            opts="-h --check --registry --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --registry)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__dict__subcmd__vendor)
            opts="-h --dir --registry --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --registry)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__explain)
            opts="-h --lang --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --lang)
                    COMPREPLY=($(compgen -W "en ja" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__fix)
            opts="-q -v -j -h --config --no-config --no-baseline --mode --exclude --no-ignore --hidden --color --quiet --verbose --stdin-filename --max-file-size --threads --no-cache --cache-dir --dry-run --interactive --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mode)
                    COMPREPLY=($(compgen -W "corrections dictionary off" -- "${cur}"))
                    return 0
                    ;;
                --exclude)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                --stdin-filename)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-file-size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --threads)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -j)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cache-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help)
            opts="check fix words dict import init config baseline explain rules completions completion-candidates lsp help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__baseline)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__check)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__completion__subcmd__candidates)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__completions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__config)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__dict)
            opts="list add search info remove update vendor"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__dict__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__dict__subcmd__info)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__dict__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__dict__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__dict__subcmd__search)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__dict__subcmd__update)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__dict__subcmd__vendor)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__explain)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__fix)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__import)
            opts="cspell typos prh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__import__subcmd__cspell)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__import__subcmd__prh)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__import__subcmd__typos)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__init)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__lsp)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__rules)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__words)
            opts="collect add triage"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__words__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__words__subcmd__collect)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__help__subcmd__words__subcmd__triage)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__import)
            opts="-h --help cspell typos prh help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__import__subcmd__cspell)
            opts="-h --dry-run --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__import__subcmd__help)
            opts="cspell typos prh help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__import__subcmd__help__subcmd__cspell)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__import__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__import__subcmd__help__subcmd__prh)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__import__subcmd__help__subcmd__typos)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__import__subcmd__prh)
            opts="-h --output --dry-run --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__import__subcmd__typos)
            opts="-h --dry-run --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__init)
            opts="-h --force --interactive --yes --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__lsp)
            opts="-h --stdio --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__rules)
            opts="-h --lang --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --lang)
                    COMPREPLY=($(compgen -W "en ja" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__words)
            opts="-h --help collect add triage help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__words__subcmd__add)
            opts="-h --global --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__words__subcmd__collect)
            opts="-h --min-count --plain --json --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --min-count)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__words__subcmd__help)
            opts="collect add triage help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__words__subcmd__help__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__words__subcmd__help__subcmd__collect)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__words__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__words__subcmd__help__subcmd__triage)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        ayame__subcmd__spell__subcmd__words__subcmd__triage)
            opts="-h --kind --min-count --limit --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --kind)
                    COMPREPLY=($(compgen -W "typo unknown-word en-variant ja-variant" -- "${cur}"))
                    return 0
                    ;;
                --min-count)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _ayame-spell -o nosort -o bashdefault -o default ayame-spell
else
    complete -F _ayame-spell -o bashdefault -o default ayame-spell
fi

# ayame-spell dynamic completion (cache-only; never performs network I/O).
_ayame_spell_dynamic_wrapper() {
    local cur="${COMP_WORDS[COMP_CWORD]}" kind=""
    if (( COMP_CWORD >= 2 )); then
        case "${COMP_WORDS[COMP_CWORD-2]} ${COMP_WORDS[COMP_CWORD-1]}" in
            "dict add") kind="dict-add" ;;
            "dict remove") kind="dict-remove" ;;
            "words add") kind="words-add" ;;
        esac
    fi
    if [[ -z "$kind" && "${COMP_WORDS[COMP_CWORD-1]}" == "--words" ]]; then
        kind="word-file"
    fi
    if [[ -n "$kind" ]]; then
        mapfile -t COMPREPLY < <(command ayame-spell __complete "$kind" "$cur")
        return
    fi
    _ayame-spell "$@"
}
complete -o bashdefault -o default -F _ayame_spell_dynamic_wrapper ayame-spell
