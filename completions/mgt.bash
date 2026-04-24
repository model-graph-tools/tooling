_mgt() {
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
                cmd="mgt"
                ;;
            mgt,analyze)
                cmd="mgt__subcmd__analyze"
                ;;
            mgt,help)
                cmd="mgt__subcmd__help"
                ;;
            mgt,neo4j)
                cmd="mgt__subcmd__neo4j"
                ;;
            mgt__subcmd__help,analyze)
                cmd="mgt__subcmd__help__subcmd__analyze"
                ;;
            mgt__subcmd__help,help)
                cmd="mgt__subcmd__help__subcmd__help"
                ;;
            mgt__subcmd__help,neo4j)
                cmd="mgt__subcmd__help__subcmd__neo4j"
                ;;
            mgt__subcmd__help__subcmd__neo4j,start)
                cmd="mgt__subcmd__help__subcmd__neo4j__subcmd__start"
                ;;
            mgt__subcmd__help__subcmd__neo4j__subcmd__start,stop)
                cmd="mgt__subcmd__help__subcmd__neo4j__subcmd__start__subcmd__stop"
                ;;
            mgt__subcmd__neo4j,help)
                cmd="mgt__subcmd__neo4j__subcmd__help"
                ;;
            mgt__subcmd__neo4j,start)
                cmd="mgt__subcmd__neo4j__subcmd__start"
                ;;
            mgt__subcmd__neo4j__subcmd__help,help)
                cmd="mgt__subcmd__neo4j__subcmd__help__subcmd__help"
                ;;
            mgt__subcmd__neo4j__subcmd__help,start)
                cmd="mgt__subcmd__neo4j__subcmd__help__subcmd__start"
                ;;
            mgt__subcmd__neo4j__subcmd__help__subcmd__start,stop)
                cmd="mgt__subcmd__neo4j__subcmd__help__subcmd__start__subcmd__stop"
                ;;
            mgt__subcmd__neo4j__subcmd__start,help)
                cmd="mgt__subcmd__neo4j__subcmd__start__subcmd__help"
                ;;
            mgt__subcmd__neo4j__subcmd__start,stop)
                cmd="mgt__subcmd__neo4j__subcmd__start__subcmd__stop"
                ;;
            mgt__subcmd__neo4j__subcmd__start__subcmd__help,help)
                cmd="mgt__subcmd__neo4j__subcmd__start__subcmd__help__subcmd__help"
                ;;
            mgt__subcmd__neo4j__subcmd__start__subcmd__help,stop)
                cmd="mgt__subcmd__neo4j__subcmd__start__subcmd__help__subcmd__stop"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        mgt)
            opts="-h -V --help --version analyze neo4j help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
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
        mgt__subcmd__analyze)
            opts="-h -V --help --version <identifier>"
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
        mgt__subcmd__help)
            opts="analyze neo4j help"
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
        mgt__subcmd__help__subcmd__analyze)
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
        mgt__subcmd__help__subcmd__help)
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
        mgt__subcmd__help__subcmd__neo4j)
            opts="start"
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
        mgt__subcmd__help__subcmd__neo4j__subcmd__start)
            opts="stop"
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
        mgt__subcmd__help__subcmd__neo4j__subcmd__start__subcmd__stop)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
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
        mgt__subcmd__neo4j)
            opts="-h -V --help --version start help"
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
        mgt__subcmd__neo4j__subcmd__help)
            opts="start help"
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
        mgt__subcmd__neo4j__subcmd__help__subcmd__help)
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
        mgt__subcmd__neo4j__subcmd__help__subcmd__start)
            opts="stop"
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
        mgt__subcmd__neo4j__subcmd__help__subcmd__start__subcmd__stop)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
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
        mgt__subcmd__neo4j__subcmd__start)
            opts="-h -V --help --version <identifier> stop help"
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
        mgt__subcmd__neo4j__subcmd__start__subcmd__help)
            opts="stop help"
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
        mgt__subcmd__neo4j__subcmd__start__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
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
        mgt__subcmd__neo4j__subcmd__start__subcmd__help__subcmd__stop)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
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
        mgt__subcmd__neo4j__subcmd__start__subcmd__stop)
            opts="-a -h -V --all --help --version [identifier]"
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
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _mgt -o nosort -o bashdefault -o default mgt
else
    complete -F _mgt -o bashdefault -o default mgt
fi
