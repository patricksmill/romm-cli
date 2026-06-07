_romm-cli() {
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
                cmd="romm__cli"
                ;;
            romm__cli,api)
                cmd="romm__cli__subcmd__api"
                ;;
            romm__cli,auth)
                cmd="romm__cli__subcmd__auth"
                ;;
            romm__cli,cache)
                cmd="romm__cli__subcmd__cache"
                ;;
            romm__cli,call)
                cmd="romm__cli__subcmd__api"
                ;;
            romm__cli,completions)
                cmd="romm__cli__subcmd__completions"
                ;;
            romm__cli,dl)
                cmd="romm__cli__subcmd__download"
                ;;
            romm__cli,download)
                cmd="romm__cli__subcmd__download"
                ;;
            romm__cli,get)
                cmd="romm__cli__subcmd__download"
                ;;
            romm__cli,help)
                cmd="romm__cli__subcmd__help"
                ;;
            romm__cli,init)
                cmd="romm__cli__subcmd__init"
                ;;
            romm__cli,p)
                cmd="romm__cli__subcmd__platforms"
                ;;
            romm__cli,platform)
                cmd="romm__cli__subcmd__platforms"
                ;;
            romm__cli,platforms)
                cmd="romm__cli__subcmd__platforms"
                ;;
            romm__cli,plats)
                cmd="romm__cli__subcmd__platforms"
                ;;
            romm__cli,r)
                cmd="romm__cli__subcmd__roms"
                ;;
            romm__cli,rom)
                cmd="romm__cli__subcmd__roms"
                ;;
            romm__cli,roms)
                cmd="romm__cli__subcmd__roms"
                ;;
            romm__cli,scan)
                cmd="romm__cli__subcmd__scan"
                ;;
            romm__cli,setup)
                cmd="romm__cli__subcmd__init"
                ;;
            romm__cli,sync)
                cmd="romm__cli__subcmd__sync"
                ;;
            romm__cli,update)
                cmd="romm__cli__subcmd__update"
                ;;
            romm__cli__subcmd__api,call)
                cmd="romm__cli__subcmd__api__subcmd__call"
                ;;
            romm__cli__subcmd__api,get)
                cmd="romm__cli__subcmd__api__subcmd__get"
                ;;
            romm__cli__subcmd__api,help)
                cmd="romm__cli__subcmd__api__subcmd__help"
                ;;
            romm__cli__subcmd__api,post)
                cmd="romm__cli__subcmd__api__subcmd__post"
                ;;
            romm__cli__subcmd__api__subcmd__help,call)
                cmd="romm__cli__subcmd__api__subcmd__help__subcmd__call"
                ;;
            romm__cli__subcmd__api__subcmd__help,get)
                cmd="romm__cli__subcmd__api__subcmd__help__subcmd__get"
                ;;
            romm__cli__subcmd__api__subcmd__help,help)
                cmd="romm__cli__subcmd__api__subcmd__help__subcmd__help"
                ;;
            romm__cli__subcmd__api__subcmd__help,post)
                cmd="romm__cli__subcmd__api__subcmd__help__subcmd__post"
                ;;
            romm__cli__subcmd__auth,help)
                cmd="romm__cli__subcmd__auth__subcmd__help"
                ;;
            romm__cli__subcmd__auth,login)
                cmd="romm__cli__subcmd__auth__subcmd__login"
                ;;
            romm__cli__subcmd__auth,logout)
                cmd="romm__cli__subcmd__auth__subcmd__logout"
                ;;
            romm__cli__subcmd__auth,status)
                cmd="romm__cli__subcmd__auth__subcmd__status"
                ;;
            romm__cli__subcmd__auth__subcmd__help,help)
                cmd="romm__cli__subcmd__auth__subcmd__help__subcmd__help"
                ;;
            romm__cli__subcmd__auth__subcmd__help,login)
                cmd="romm__cli__subcmd__auth__subcmd__help__subcmd__login"
                ;;
            romm__cli__subcmd__auth__subcmd__help,logout)
                cmd="romm__cli__subcmd__auth__subcmd__help__subcmd__logout"
                ;;
            romm__cli__subcmd__auth__subcmd__help,status)
                cmd="romm__cli__subcmd__auth__subcmd__help__subcmd__status"
                ;;
            romm__cli__subcmd__cache,clear)
                cmd="romm__cli__subcmd__cache__subcmd__clear"
                ;;
            romm__cli__subcmd__cache,help)
                cmd="romm__cli__subcmd__cache__subcmd__help"
                ;;
            romm__cli__subcmd__cache,info)
                cmd="romm__cli__subcmd__cache__subcmd__info"
                ;;
            romm__cli__subcmd__cache,path)
                cmd="romm__cli__subcmd__cache__subcmd__path"
                ;;
            romm__cli__subcmd__cache__subcmd__help,clear)
                cmd="romm__cli__subcmd__cache__subcmd__help__subcmd__clear"
                ;;
            romm__cli__subcmd__cache__subcmd__help,help)
                cmd="romm__cli__subcmd__cache__subcmd__help__subcmd__help"
                ;;
            romm__cli__subcmd__cache__subcmd__help,info)
                cmd="romm__cli__subcmd__cache__subcmd__help__subcmd__info"
                ;;
            romm__cli__subcmd__cache__subcmd__help,path)
                cmd="romm__cli__subcmd__cache__subcmd__help__subcmd__path"
                ;;
            romm__cli__subcmd__download,all)
                cmd="romm__cli__subcmd__download__subcmd__batch"
                ;;
            romm__cli__subcmd__download,batch)
                cmd="romm__cli__subcmd__download__subcmd__batch"
                ;;
            romm__cli__subcmd__download,extras)
                cmd="romm__cli__subcmd__download__subcmd__extras"
                ;;
            romm__cli__subcmd__download,help)
                cmd="romm__cli__subcmd__download__subcmd__help"
                ;;
            romm__cli__subcmd__download__subcmd__help,batch)
                cmd="romm__cli__subcmd__download__subcmd__help__subcmd__batch"
                ;;
            romm__cli__subcmd__download__subcmd__help,extras)
                cmd="romm__cli__subcmd__download__subcmd__help__subcmd__extras"
                ;;
            romm__cli__subcmd__download__subcmd__help,help)
                cmd="romm__cli__subcmd__download__subcmd__help__subcmd__help"
                ;;
            romm__cli__subcmd__help,api)
                cmd="romm__cli__subcmd__help__subcmd__api"
                ;;
            romm__cli__subcmd__help,auth)
                cmd="romm__cli__subcmd__help__subcmd__auth"
                ;;
            romm__cli__subcmd__help,cache)
                cmd="romm__cli__subcmd__help__subcmd__cache"
                ;;
            romm__cli__subcmd__help,completions)
                cmd="romm__cli__subcmd__help__subcmd__completions"
                ;;
            romm__cli__subcmd__help,download)
                cmd="romm__cli__subcmd__help__subcmd__download"
                ;;
            romm__cli__subcmd__help,help)
                cmd="romm__cli__subcmd__help__subcmd__help"
                ;;
            romm__cli__subcmd__help,init)
                cmd="romm__cli__subcmd__help__subcmd__init"
                ;;
            romm__cli__subcmd__help,platforms)
                cmd="romm__cli__subcmd__help__subcmd__platforms"
                ;;
            romm__cli__subcmd__help,roms)
                cmd="romm__cli__subcmd__help__subcmd__roms"
                ;;
            romm__cli__subcmd__help,scan)
                cmd="romm__cli__subcmd__help__subcmd__scan"
                ;;
            romm__cli__subcmd__help,sync)
                cmd="romm__cli__subcmd__help__subcmd__sync"
                ;;
            romm__cli__subcmd__help,update)
                cmd="romm__cli__subcmd__help__subcmd__update"
                ;;
            romm__cli__subcmd__help__subcmd__api,call)
                cmd="romm__cli__subcmd__help__subcmd__api__subcmd__call"
                ;;
            romm__cli__subcmd__help__subcmd__api,get)
                cmd="romm__cli__subcmd__help__subcmd__api__subcmd__get"
                ;;
            romm__cli__subcmd__help__subcmd__api,post)
                cmd="romm__cli__subcmd__help__subcmd__api__subcmd__post"
                ;;
            romm__cli__subcmd__help__subcmd__auth,login)
                cmd="romm__cli__subcmd__help__subcmd__auth__subcmd__login"
                ;;
            romm__cli__subcmd__help__subcmd__auth,logout)
                cmd="romm__cli__subcmd__help__subcmd__auth__subcmd__logout"
                ;;
            romm__cli__subcmd__help__subcmd__auth,status)
                cmd="romm__cli__subcmd__help__subcmd__auth__subcmd__status"
                ;;
            romm__cli__subcmd__help__subcmd__cache,clear)
                cmd="romm__cli__subcmd__help__subcmd__cache__subcmd__clear"
                ;;
            romm__cli__subcmd__help__subcmd__cache,info)
                cmd="romm__cli__subcmd__help__subcmd__cache__subcmd__info"
                ;;
            romm__cli__subcmd__help__subcmd__cache,path)
                cmd="romm__cli__subcmd__help__subcmd__cache__subcmd__path"
                ;;
            romm__cli__subcmd__help__subcmd__download,batch)
                cmd="romm__cli__subcmd__help__subcmd__download__subcmd__batch"
                ;;
            romm__cli__subcmd__help__subcmd__download,extras)
                cmd="romm__cli__subcmd__help__subcmd__download__subcmd__extras"
                ;;
            romm__cli__subcmd__help__subcmd__platforms,get)
                cmd="romm__cli__subcmd__help__subcmd__platforms__subcmd__get"
                ;;
            romm__cli__subcmd__help__subcmd__platforms,list)
                cmd="romm__cli__subcmd__help__subcmd__platforms__subcmd__list"
                ;;
            romm__cli__subcmd__help__subcmd__roms,cover-search)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__cover__subcmd__search"
                ;;
            romm__cli__subcmd__help__subcmd__roms,delete)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__delete"
                ;;
            romm__cli__subcmd__help__subcmd__roms,filters)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__filters"
                ;;
            romm__cli__subcmd__help__subcmd__roms,find)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__find"
                ;;
            romm__cli__subcmd__help__subcmd__roms,get)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__get"
                ;;
            romm__cli__subcmd__help__subcmd__roms,manuals-add)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__manuals__subcmd__add"
                ;;
            romm__cli__subcmd__help__subcmd__roms,notes-add)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__notes__subcmd__add"
                ;;
            romm__cli__subcmd__help__subcmd__roms,notes-delete)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__notes__subcmd__delete"
                ;;
            romm__cli__subcmd__help__subcmd__roms,notes-list)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__notes__subcmd__list"
                ;;
            romm__cli__subcmd__help__subcmd__roms,notes-update)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__notes__subcmd__update"
                ;;
            romm__cli__subcmd__help__subcmd__roms,props)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__props"
                ;;
            romm__cli__subcmd__help__subcmd__roms,upload)
                cmd="romm__cli__subcmd__help__subcmd__roms__subcmd__upload"
                ;;
            romm__cli__subcmd__help__subcmd__sync,device)
                cmd="romm__cli__subcmd__help__subcmd__sync__subcmd__device"
                ;;
            romm__cli__subcmd__help__subcmd__sync,plan)
                cmd="romm__cli__subcmd__help__subcmd__sync__subcmd__plan"
                ;;
            romm__cli__subcmd__help__subcmd__sync,push-pull)
                cmd="romm__cli__subcmd__help__subcmd__sync__subcmd__push__subcmd__pull"
                ;;
            romm__cli__subcmd__help__subcmd__sync,run)
                cmd="romm__cli__subcmd__help__subcmd__sync__subcmd__run"
                ;;
            romm__cli__subcmd__help__subcmd__sync,sessions)
                cmd="romm__cli__subcmd__help__subcmd__sync__subcmd__sessions"
                ;;
            romm__cli__subcmd__help__subcmd__sync__subcmd__device,get)
                cmd="romm__cli__subcmd__help__subcmd__sync__subcmd__device__subcmd__get"
                ;;
            romm__cli__subcmd__help__subcmd__sync__subcmd__device,list)
                cmd="romm__cli__subcmd__help__subcmd__sync__subcmd__device__subcmd__list"
                ;;
            romm__cli__subcmd__help__subcmd__sync__subcmd__device,register)
                cmd="romm__cli__subcmd__help__subcmd__sync__subcmd__device__subcmd__register"
                ;;
            romm__cli__subcmd__help__subcmd__sync__subcmd__sessions,get)
                cmd="romm__cli__subcmd__help__subcmd__sync__subcmd__sessions__subcmd__get"
                ;;
            romm__cli__subcmd__help__subcmd__sync__subcmd__sessions,list)
                cmd="romm__cli__subcmd__help__subcmd__sync__subcmd__sessions__subcmd__list"
                ;;
            romm__cli__subcmd__platforms,get)
                cmd="romm__cli__subcmd__platforms__subcmd__get"
                ;;
            romm__cli__subcmd__platforms,help)
                cmd="romm__cli__subcmd__platforms__subcmd__help"
                ;;
            romm__cli__subcmd__platforms,info)
                cmd="romm__cli__subcmd__platforms__subcmd__get"
                ;;
            romm__cli__subcmd__platforms,list)
                cmd="romm__cli__subcmd__platforms__subcmd__list"
                ;;
            romm__cli__subcmd__platforms,ls)
                cmd="romm__cli__subcmd__platforms__subcmd__list"
                ;;
            romm__cli__subcmd__platforms__subcmd__help,get)
                cmd="romm__cli__subcmd__platforms__subcmd__help__subcmd__get"
                ;;
            romm__cli__subcmd__platforms__subcmd__help,help)
                cmd="romm__cli__subcmd__platforms__subcmd__help__subcmd__help"
                ;;
            romm__cli__subcmd__platforms__subcmd__help,list)
                cmd="romm__cli__subcmd__platforms__subcmd__help__subcmd__list"
                ;;
            romm__cli__subcmd__roms,cover-search)
                cmd="romm__cli__subcmd__roms__subcmd__cover__subcmd__search"
                ;;
            romm__cli__subcmd__roms,delete)
                cmd="romm__cli__subcmd__roms__subcmd__delete"
                ;;
            romm__cli__subcmd__roms,filters)
                cmd="romm__cli__subcmd__roms__subcmd__filters"
                ;;
            romm__cli__subcmd__roms,find)
                cmd="romm__cli__subcmd__roms__subcmd__find"
                ;;
            romm__cli__subcmd__roms,get)
                cmd="romm__cli__subcmd__roms__subcmd__get"
                ;;
            romm__cli__subcmd__roms,help)
                cmd="romm__cli__subcmd__roms__subcmd__help"
                ;;
            romm__cli__subcmd__roms,info)
                cmd="romm__cli__subcmd__roms__subcmd__get"
                ;;
            romm__cli__subcmd__roms,manuals-add)
                cmd="romm__cli__subcmd__roms__subcmd__manuals__subcmd__add"
                ;;
            romm__cli__subcmd__roms,notes-add)
                cmd="romm__cli__subcmd__roms__subcmd__notes__subcmd__add"
                ;;
            romm__cli__subcmd__roms,notes-delete)
                cmd="romm__cli__subcmd__roms__subcmd__notes__subcmd__delete"
                ;;
            romm__cli__subcmd__roms,notes-list)
                cmd="romm__cli__subcmd__roms__subcmd__notes__subcmd__list"
                ;;
            romm__cli__subcmd__roms,notes-update)
                cmd="romm__cli__subcmd__roms__subcmd__notes__subcmd__update"
                ;;
            romm__cli__subcmd__roms,props)
                cmd="romm__cli__subcmd__roms__subcmd__props"
                ;;
            romm__cli__subcmd__roms,up)
                cmd="romm__cli__subcmd__roms__subcmd__upload"
                ;;
            romm__cli__subcmd__roms,upload)
                cmd="romm__cli__subcmd__roms__subcmd__upload"
                ;;
            romm__cli__subcmd__roms__subcmd__help,cover-search)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__cover__subcmd__search"
                ;;
            romm__cli__subcmd__roms__subcmd__help,delete)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__delete"
                ;;
            romm__cli__subcmd__roms__subcmd__help,filters)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__filters"
                ;;
            romm__cli__subcmd__roms__subcmd__help,find)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__find"
                ;;
            romm__cli__subcmd__roms__subcmd__help,get)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__get"
                ;;
            romm__cli__subcmd__roms__subcmd__help,help)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__help"
                ;;
            romm__cli__subcmd__roms__subcmd__help,manuals-add)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__manuals__subcmd__add"
                ;;
            romm__cli__subcmd__roms__subcmd__help,notes-add)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__notes__subcmd__add"
                ;;
            romm__cli__subcmd__roms__subcmd__help,notes-delete)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__notes__subcmd__delete"
                ;;
            romm__cli__subcmd__roms__subcmd__help,notes-list)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__notes__subcmd__list"
                ;;
            romm__cli__subcmd__roms__subcmd__help,notes-update)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__notes__subcmd__update"
                ;;
            romm__cli__subcmd__roms__subcmd__help,props)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__props"
                ;;
            romm__cli__subcmd__roms__subcmd__help,upload)
                cmd="romm__cli__subcmd__roms__subcmd__help__subcmd__upload"
                ;;
            romm__cli__subcmd__sync,device)
                cmd="romm__cli__subcmd__sync__subcmd__device"
                ;;
            romm__cli__subcmd__sync,help)
                cmd="romm__cli__subcmd__sync__subcmd__help"
                ;;
            romm__cli__subcmd__sync,plan)
                cmd="romm__cli__subcmd__sync__subcmd__plan"
                ;;
            romm__cli__subcmd__sync,push-pull)
                cmd="romm__cli__subcmd__sync__subcmd__push__subcmd__pull"
                ;;
            romm__cli__subcmd__sync,run)
                cmd="romm__cli__subcmd__sync__subcmd__run"
                ;;
            romm__cli__subcmd__sync,sessions)
                cmd="romm__cli__subcmd__sync__subcmd__sessions"
                ;;
            romm__cli__subcmd__sync__subcmd__device,get)
                cmd="romm__cli__subcmd__sync__subcmd__device__subcmd__get"
                ;;
            romm__cli__subcmd__sync__subcmd__device,help)
                cmd="romm__cli__subcmd__sync__subcmd__device__subcmd__help"
                ;;
            romm__cli__subcmd__sync__subcmd__device,list)
                cmd="romm__cli__subcmd__sync__subcmd__device__subcmd__list"
                ;;
            romm__cli__subcmd__sync__subcmd__device,register)
                cmd="romm__cli__subcmd__sync__subcmd__device__subcmd__register"
                ;;
            romm__cli__subcmd__sync__subcmd__device__subcmd__help,get)
                cmd="romm__cli__subcmd__sync__subcmd__device__subcmd__help__subcmd__get"
                ;;
            romm__cli__subcmd__sync__subcmd__device__subcmd__help,help)
                cmd="romm__cli__subcmd__sync__subcmd__device__subcmd__help__subcmd__help"
                ;;
            romm__cli__subcmd__sync__subcmd__device__subcmd__help,list)
                cmd="romm__cli__subcmd__sync__subcmd__device__subcmd__help__subcmd__list"
                ;;
            romm__cli__subcmd__sync__subcmd__device__subcmd__help,register)
                cmd="romm__cli__subcmd__sync__subcmd__device__subcmd__help__subcmd__register"
                ;;
            romm__cli__subcmd__sync__subcmd__help,device)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__device"
                ;;
            romm__cli__subcmd__sync__subcmd__help,help)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__help"
                ;;
            romm__cli__subcmd__sync__subcmd__help,plan)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__plan"
                ;;
            romm__cli__subcmd__sync__subcmd__help,push-pull)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__push__subcmd__pull"
                ;;
            romm__cli__subcmd__sync__subcmd__help,run)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__run"
                ;;
            romm__cli__subcmd__sync__subcmd__help,sessions)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__sessions"
                ;;
            romm__cli__subcmd__sync__subcmd__help__subcmd__device,get)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__device__subcmd__get"
                ;;
            romm__cli__subcmd__sync__subcmd__help__subcmd__device,list)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__device__subcmd__list"
                ;;
            romm__cli__subcmd__sync__subcmd__help__subcmd__device,register)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__device__subcmd__register"
                ;;
            romm__cli__subcmd__sync__subcmd__help__subcmd__sessions,get)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__sessions__subcmd__get"
                ;;
            romm__cli__subcmd__sync__subcmd__help__subcmd__sessions,list)
                cmd="romm__cli__subcmd__sync__subcmd__help__subcmd__sessions__subcmd__list"
                ;;
            romm__cli__subcmd__sync__subcmd__sessions,get)
                cmd="romm__cli__subcmd__sync__subcmd__sessions__subcmd__get"
                ;;
            romm__cli__subcmd__sync__subcmd__sessions,help)
                cmd="romm__cli__subcmd__sync__subcmd__sessions__subcmd__help"
                ;;
            romm__cli__subcmd__sync__subcmd__sessions,list)
                cmd="romm__cli__subcmd__sync__subcmd__sessions__subcmd__list"
                ;;
            romm__cli__subcmd__sync__subcmd__sessions__subcmd__help,get)
                cmd="romm__cli__subcmd__sync__subcmd__sessions__subcmd__help__subcmd__get"
                ;;
            romm__cli__subcmd__sync__subcmd__sessions__subcmd__help,help)
                cmd="romm__cli__subcmd__sync__subcmd__sessions__subcmd__help__subcmd__help"
                ;;
            romm__cli__subcmd__sync__subcmd__sessions__subcmd__help,list)
                cmd="romm__cli__subcmd__sync__subcmd__sessions__subcmd__help__subcmd__list"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        romm__cli)
            opts="-v -h -V --verbose --json --help --version init setup api call platforms platform p plats roms rom r scan sync download dl get cache auth update completions help"
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
        romm__subcmd__cli__subcmd__api)
            opts="-v -h --query --data --verbose --json --help [METHOD] [PATH] call get post help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --query)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --data)
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
        romm__subcmd__cli__subcmd__api__subcmd__call)
            opts="-v -h --query --data --verbose --json --help <METHOD> <PATH>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --query)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --data)
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
        romm__subcmd__cli__subcmd__api__subcmd__get)
            opts="-v -h --query --data --verbose --json --help <PATH>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --query)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --data)
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
        romm__subcmd__cli__subcmd__api__subcmd__help)
            opts="call get post help"
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
        romm__subcmd__cli__subcmd__api__subcmd__help__subcmd__call)
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
        romm__subcmd__cli__subcmd__api__subcmd__help__subcmd__get)
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
        romm__subcmd__cli__subcmd__api__subcmd__help__subcmd__help)
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
        romm__subcmd__cli__subcmd__api__subcmd__help__subcmd__post)
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
        romm__subcmd__cli__subcmd__api__subcmd__post)
            opts="-v -h --query --data --verbose --json --help <PATH>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --query)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --data)
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
        romm__subcmd__cli__subcmd__auth)
            opts="-v -h --verbose --json --help login logout status help"
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
        romm__subcmd__cli__subcmd__auth__subcmd__help)
            opts="login logout status help"
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
        romm__subcmd__cli__subcmd__auth__subcmd__help__subcmd__help)
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
        romm__subcmd__cli__subcmd__auth__subcmd__help__subcmd__login)
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
        romm__subcmd__cli__subcmd__auth__subcmd__help__subcmd__logout)
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
        romm__subcmd__cli__subcmd__auth__subcmd__help__subcmd__status)
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
        romm__subcmd__cli__subcmd__auth__subcmd__login)
            opts="-v -h --token --token-file --username --password --password-file --api-key-header --api-key --pairing-code --verbose --json --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --token)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --token-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --username)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --password-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api-key-header)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api-key)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pairing-code)
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
        romm__subcmd__cli__subcmd__auth__subcmd__logout)
            opts="-v -h --verbose --json --help"
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
        romm__subcmd__cli__subcmd__auth__subcmd__status)
            opts="-v -h --verbose --json --help"
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
        romm__subcmd__cli__subcmd__cache)
            opts="-v -h --verbose --json --help path info clear help"
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
        romm__subcmd__cli__subcmd__cache__subcmd__clear)
            opts="-v -h --verbose --json --help"
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
        romm__subcmd__cli__subcmd__cache__subcmd__help)
            opts="path info clear help"
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
        romm__subcmd__cli__subcmd__cache__subcmd__help__subcmd__clear)
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
        romm__subcmd__cli__subcmd__cache__subcmd__help__subcmd__help)
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
        romm__subcmd__cli__subcmd__cache__subcmd__help__subcmd__info)
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
        romm__subcmd__cli__subcmd__cache__subcmd__help__subcmd__path)
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
        romm__subcmd__cli__subcmd__cache__subcmd__info)
            opts="-v -h --verbose --json --help"
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
        romm__subcmd__cli__subcmd__cache__subcmd__path)
            opts="-v -h --verbose --json --help"
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
        romm__subcmd__cli__subcmd__completions)
            opts="-v -h --verbose --json --help bash elvish fish powershell zsh"
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
        romm__subcmd__cli__subcmd__download)
            opts="-o -y -v -h --output --platform --search-term --jobs --extract --extract-layout --delete-zip-after-extract --with-extras --no-extras --yes --verbose --json --help [ROM_ID] batch all extras help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --search-term)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --jobs)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --extract-layout)
                    COMPREPLY=($(compgen -W "platform flat rom" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        romm__subcmd__cli__subcmd__download__subcmd__batch)
            opts="-o -y -v -h --output --platform --search-term --jobs --extract --extract-layout --delete-zip-after-extract --with-extras --no-extras --yes --verbose --json --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --search-term)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --jobs)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --extract-layout)
                    COMPREPLY=($(compgen -W "platform flat rom" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        romm__subcmd__cli__subcmd__download__subcmd__extras)
            opts="-o -y -v -h --output --platform --search-term --jobs --extract --extract-layout --delete-zip-after-extract --with-extras --no-extras --yes --verbose --json --help <ROM_ID>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --search-term)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --jobs)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --extract-layout)
                    COMPREPLY=($(compgen -W "platform flat rom" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        romm__subcmd__cli__subcmd__download__subcmd__help)
            opts="batch extras help"
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
        romm__subcmd__cli__subcmd__download__subcmd__help__subcmd__batch)
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
        romm__subcmd__cli__subcmd__download__subcmd__help__subcmd__extras)
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
        romm__subcmd__cli__subcmd__download__subcmd__help__subcmd__help)
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
        romm__subcmd__cli__subcmd__help)
            opts="init api platforms roms scan sync download cache auth update completions help"
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
        romm__subcmd__cli__subcmd__help__subcmd__api)
            opts="call get post"
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
        romm__subcmd__cli__subcmd__help__subcmd__api__subcmd__call)
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
        romm__subcmd__cli__subcmd__help__subcmd__api__subcmd__get)
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
        romm__subcmd__cli__subcmd__help__subcmd__api__subcmd__post)
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
        romm__subcmd__cli__subcmd__help__subcmd__auth)
            opts="login logout status"
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
        romm__subcmd__cli__subcmd__help__subcmd__auth__subcmd__login)
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
        romm__subcmd__cli__subcmd__help__subcmd__auth__subcmd__logout)
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
        romm__subcmd__cli__subcmd__help__subcmd__auth__subcmd__status)
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
        romm__subcmd__cli__subcmd__help__subcmd__cache)
            opts="path info clear"
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
        romm__subcmd__cli__subcmd__help__subcmd__cache__subcmd__clear)
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
        romm__subcmd__cli__subcmd__help__subcmd__cache__subcmd__info)
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
        romm__subcmd__cli__subcmd__help__subcmd__cache__subcmd__path)
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
        romm__subcmd__cli__subcmd__help__subcmd__completions)
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
        romm__subcmd__cli__subcmd__help__subcmd__download)
            opts="batch extras"
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
        romm__subcmd__cli__subcmd__help__subcmd__download__subcmd__batch)
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
        romm__subcmd__cli__subcmd__help__subcmd__download__subcmd__extras)
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
        romm__subcmd__cli__subcmd__help__subcmd__help)
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
        romm__subcmd__cli__subcmd__help__subcmd__init)
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
        romm__subcmd__cli__subcmd__help__subcmd__platforms)
            opts="list get"
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
        romm__subcmd__cli__subcmd__help__subcmd__platforms__subcmd__get)
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
        romm__subcmd__cli__subcmd__help__subcmd__platforms__subcmd__list)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms)
            opts="get find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload"
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__cover__subcmd__search)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__delete)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__filters)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__find)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__get)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__manuals__subcmd__add)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__notes__subcmd__add)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__notes__subcmd__delete)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__notes__subcmd__list)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__notes__subcmd__update)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__props)
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
        romm__subcmd__cli__subcmd__help__subcmd__roms__subcmd__upload)
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
        romm__subcmd__cli__subcmd__help__subcmd__scan)
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
        romm__subcmd__cli__subcmd__help__subcmd__sync)
            opts="device plan run sessions push-pull"
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
        romm__subcmd__cli__subcmd__help__subcmd__sync__subcmd__device)
            opts="register list get"
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
        romm__subcmd__cli__subcmd__help__subcmd__sync__subcmd__device__subcmd__get)
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
        romm__subcmd__cli__subcmd__help__subcmd__sync__subcmd__device__subcmd__list)
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
        romm__subcmd__cli__subcmd__help__subcmd__sync__subcmd__device__subcmd__register)
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
        romm__subcmd__cli__subcmd__help__subcmd__sync__subcmd__plan)
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
        romm__subcmd__cli__subcmd__help__subcmd__sync__subcmd__push__subcmd__pull)
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
        romm__subcmd__cli__subcmd__help__subcmd__sync__subcmd__run)
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
        romm__subcmd__cli__subcmd__help__subcmd__sync__subcmd__sessions)
            opts="list get"
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
        romm__subcmd__cli__subcmd__help__subcmd__sync__subcmd__sessions__subcmd__get)
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
        romm__subcmd__cli__subcmd__help__subcmd__sync__subcmd__sessions__subcmd__list)
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
        romm__subcmd__cli__subcmd__help__subcmd__update)
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
        romm__subcmd__cli__subcmd__init)
            opts="-v -h --force --print-path --url --token --token-file --download-dir --no-https --check --verbose --json --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --url)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --token)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --token-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --download-dir)
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
        romm__subcmd__cli__subcmd__platforms)
            opts="-v -h --json --verbose --help list ls get info help"
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
        romm__subcmd__cli__subcmd__platforms__subcmd__get)
            opts="-v -h --json --verbose --help <ID>"
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
        romm__subcmd__cli__subcmd__platforms__subcmd__help)
            opts="list get help"
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
        romm__subcmd__cli__subcmd__platforms__subcmd__help__subcmd__get)
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
        romm__subcmd__cli__subcmd__platforms__subcmd__help__subcmd__help)
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
        romm__subcmd__cli__subcmd__platforms__subcmd__help__subcmd__list)
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
        romm__subcmd__cli__subcmd__platforms__subcmd__list)
            opts="-v -h --json --verbose --help"
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
        romm__subcmd__cli__subcmd__roms)
            opts="-v -h --json --query --q --search-term --p --platform --collection --smart-collection --virtual-collection --limit --offset --matched --favorite --duplicate --last-played --playable --missing --has-ra --verified --group-by-meta-id --with-char-index --with-filter-values --genre --franchise --collection-tag --company --age-rating --status --region --language --player-count --genres-logic --franchises-logic --collections-logic --companies-logic --age-ratings-logic --regions-logic --languages-logic --statuses-logic --player-counts-logic --order-by --order-dir --updated-after --verbose --help get info find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload up help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --search-term)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --query)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --q)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --collection)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --smart-collection)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --virtual-collection)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --offset)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --matched)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --favorite)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --duplicate)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --last-played)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --playable)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --missing)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --has-ra)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --verified)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --group-by-meta-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --with-char-index)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --with-filter-values)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --genre)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --franchise)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --collection-tag)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --company)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --age-rating)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --status)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --region)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --language)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --player-count)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --genres-logic)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --franchises-logic)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --collections-logic)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --companies-logic)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --age-ratings-logic)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --regions-logic)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --languages-logic)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --statuses-logic)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --player-counts-logic)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --order-by)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --order-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --updated-after)
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
        romm__subcmd__cli__subcmd__roms__subcmd__cover__subcmd__search)
            opts="-v -h --query --search-by --json --verbose --help <ROM_ID>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --query)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --search-by)
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
        romm__subcmd__cli__subcmd__roms__subcmd__delete)
            opts="-v -h --delete-from-fs --yes --json --verbose --help <ROM_IDS>..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --delete-from-fs)
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
        romm__subcmd__cli__subcmd__roms__subcmd__filters)
            opts="-v -h --json --verbose --help"
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
        romm__subcmd__cli__subcmd__roms__subcmd__find)
            opts="-v -h --crc --md5 --sha1 --igdb-id --moby-id --ss-id --ra-id --launchbox-id --hasheous-id --tgdb-id --flashpoint-id --hltb-id --json --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --crc)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --md5)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --sha1)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --igdb-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --moby-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ss-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ra-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --launchbox-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hasheous-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tgdb-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --flashpoint-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hltb-id)
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
        romm__subcmd__cli__subcmd__roms__subcmd__get)
            opts="-v -h --json --verbose --help <ID>"
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
        romm__subcmd__cli__subcmd__roms__subcmd__help)
            opts="get find filters delete props notes-list notes-add notes-update notes-delete manuals-add cover-search upload help"
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__cover__subcmd__search)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__delete)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__filters)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__find)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__get)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__help)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__manuals__subcmd__add)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__notes__subcmd__add)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__notes__subcmd__delete)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__notes__subcmd__list)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__notes__subcmd__update)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__props)
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
        romm__subcmd__cli__subcmd__roms__subcmd__help__subcmd__upload)
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
        romm__subcmd__cli__subcmd__roms__subcmd__manuals__subcmd__add)
            opts="-v -h --json --verbose --help <ROM_ID> <FILE>"
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
        romm__subcmd__cli__subcmd__roms__subcmd__notes__subcmd__add)
            opts="-v -h --json --verbose --help <ROM_ID>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --json)
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
        romm__subcmd__cli__subcmd__roms__subcmd__notes__subcmd__delete)
            opts="-v -h --json --verbose --help <ROM_ID> <NOTE_ID>"
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
        romm__subcmd__cli__subcmd__roms__subcmd__notes__subcmd__list)
            opts="-v -h --public-only --search --tag --json --verbose --help <ROM_ID>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --public-only)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --search)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tag)
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
        romm__subcmd__cli__subcmd__roms__subcmd__notes__subcmd__update)
            opts="-v -h --json --verbose --help <ROM_ID> <NOTE_ID>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --json)
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
        romm__subcmd__cli__subcmd__roms__subcmd__props)
            opts="-v -h --is-main-sibling --backlogged --now-playing --hidden --rating --difficulty --completion --status --update-last-played --remove-last-played --json --verbose --help <ID>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --is-main-sibling)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --backlogged)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --now-playing)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hidden)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --rating)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --difficulty)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --completion)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --status)
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
        romm__subcmd__cli__subcmd__roms__subcmd__upload)
            opts="-s -v -h --platform --scan --wait --wait-timeout-secs --json --verbose --help <FILE>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --wait-timeout-secs)
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
        romm__subcmd__cli__subcmd__scan)
            opts="-v -h --platform --wait --wait-timeout-secs --verbose --json --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --platform)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --wait-timeout-secs)
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
        romm__subcmd__cli__subcmd__sync)
            opts="-v -h --json --verbose --help device plan run sessions push-pull help"
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
        romm__subcmd__cli__subcmd__sync__subcmd__device)
            opts="-v -h --json --verbose --help register list get help"
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
        romm__subcmd__cli__subcmd__sync__subcmd__device__subcmd__get)
            opts="-v -h --json --verbose --help <DEVICE_ID>"
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
        romm__subcmd__cli__subcmd__sync__subcmd__device__subcmd__help)
            opts="register list get help"
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
        romm__subcmd__cli__subcmd__sync__subcmd__device__subcmd__help__subcmd__get)
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
        romm__subcmd__cli__subcmd__sync__subcmd__device__subcmd__help__subcmd__help)
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
        romm__subcmd__cli__subcmd__sync__subcmd__device__subcmd__help__subcmd__list)
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
        romm__subcmd__cli__subcmd__sync__subcmd__device__subcmd__help__subcmd__register)
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
        romm__subcmd__cli__subcmd__sync__subcmd__device__subcmd__list)
            opts="-v -h --json --verbose --help"
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
        romm__subcmd__cli__subcmd__sync__subcmd__device__subcmd__register)
            opts="-v -h --name --platform --client --client-version --hostname --mac-address --ip-address --sync-mode --sync-config-json --allow-duplicate --reset-syncs --json --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --platform)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --client)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --client-version)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hostname)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mac-address)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ip-address)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --sync-mode)
                    COMPREPLY=($(compgen -W "api file-transfer push-pull" -- "${cur}"))
                    return 0
                    ;;
                --sync-config-json)
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
        romm__subcmd__cli__subcmd__sync__subcmd__help)
            opts="device plan run sessions push-pull help"
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__device)
            opts="register list get"
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__device__subcmd__get)
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__device__subcmd__list)
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__device__subcmd__register)
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__help)
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__plan)
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__push__subcmd__pull)
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__run)
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__sessions)
            opts="list get"
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__sessions__subcmd__get)
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
        romm__subcmd__cli__subcmd__sync__subcmd__help__subcmd__sessions__subcmd__list)
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
        romm__subcmd__cli__subcmd__sync__subcmd__plan)
            opts="-v -h --device-id --manifest --json --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --device-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --manifest)
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
        romm__subcmd__cli__subcmd__sync__subcmd__push__subcmd__pull)
            opts="-v -h --json --verbose --help <DEVICE_ID>"
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
        romm__subcmd__cli__subcmd__sync__subcmd__run)
            opts="-v -h --device-id --manifest --download-dir --conflict --json --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --device-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --manifest)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --download-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --conflict)
                    COMPREPLY=($(compgen -W "fail skip" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        romm__subcmd__cli__subcmd__sync__subcmd__sessions)
            opts="-v -h --json --verbose --help list get help"
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
        romm__subcmd__cli__subcmd__sync__subcmd__sessions__subcmd__get)
            opts="-v -h --json --verbose --help <SESSION_ID>"
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
        romm__subcmd__cli__subcmd__sync__subcmd__sessions__subcmd__help)
            opts="list get help"
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
        romm__subcmd__cli__subcmd__sync__subcmd__sessions__subcmd__help__subcmd__get)
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
        romm__subcmd__cli__subcmd__sync__subcmd__sessions__subcmd__help__subcmd__help)
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
        romm__subcmd__cli__subcmd__sync__subcmd__sessions__subcmd__help__subcmd__list)
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
        romm__subcmd__cli__subcmd__sync__subcmd__sessions__subcmd__list)
            opts="-v -h --device-id --limit --json --verbose --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --device-id)
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
        romm__subcmd__cli__subcmd__update)
            opts="-v -h --verbose --json --help"
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
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _romm-cli -o nosort -o bashdefault -o default romm-cli
else
    complete -F _romm-cli -o bashdefault -o default romm-cli
fi
