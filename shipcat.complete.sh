#!/usr/bin/env bash

# shipcat(1) completion
_shipcat()
{
    # shellcheck disable=2034
    local cur prev words cword
    if declare -F _init_completions >/dev/null 2>&1; then
        _init_completion
    else
        _get_comp_words_by_ref cur prev words cword
    fi

    local -r subcommands="help validate shell logs graph get helm cluster gdpr
                          kong jenkins list-regions list-services"

    local has_sub
    for (( i=0; i < ${#words[@]}-1; i++ )); do
        if [[ ${words[i]} == @(help|validate|status|shell|logs|get|graph|cluster|helm|gdpr|kong|jenkins) ]]; then
            has_sub=1
        fi
    done

    # global flags
    if [[ $prev = 'shipcat' && "$cur" == -* ]]; then
        COMPREPLY=( $(compgen -W '-v -h -V --version --help' -- "$cur" ) )
        return 0
    fi
    # first subcommand
    if [[ -z "$has_sub" ]]; then
        COMPREPLY=( $(compgen -W "$subcommands" -- "$cur" ) )
        return 0
    fi

    # special subcommand completions
    local special i
    for (( i=0; i < ${#words[@]}-1; i++ )); do
        if [[ ${words[i]} == @(validate|shell|logs|graph|get|cluster|helm|list-services|gdpr|kong|jenkins) ]]; then
            special=${words[i]}
        fi
    done
    # TODO: fix cluster -> helm swap :(

    if [[ -n $special ]]; then
        case $special in
            get)
                local -r regions="$(shipcat list-regions)"
                local -r resources="version ver image"
                if [[ $prev == @(-r|--region) ]]; then
                    COMPREPLY=($(compgen -W "$regions" -- "$cur"))
                else
                    COMPREPLY=($(compgen -W "$resources" -- "$cur"))
                fi
                ;;
            gdpr)
                local -r region="$(kubectl config current-context)"
                local -r svcs="$(shipcat list-services "$region")"
                COMPREPLY=($(compgen -W "$svcs" -- "$cur"))
                ;;
            kong)
                COMPREPLY=($(compgen -W "config-url" -- "$cur"))
                ;;
            list-services)
                local -r regions="$(shipcat list-regions)"
                COMPREPLY=($(compgen -W "$regions" -- "$cur"))
                ;;
            validate|graph)
                local -r regions="$(shipcat list-regions)"
                if [[ $prev = @(graph|validate) ]]; then
                    svcs=$(find "./services" -maxdepth 1 -mindepth 1 -type d -printf "%f " 2> /dev/null)
                    COMPREPLY=($(compgen -W "$svcs -r --region" -- "$cur"))
                elif [[ $prev == @(-r|--region) ]]; then
                    COMPREPLY=($(compgen -W "$regions" -- "$cur"))
                else
                    # Identify which region we used
                    local region i
                    for (( i=2; i < ${#words[@]}-1; i++ )); do
                        if [[ ${words[i]} != -* ]] && echo "$regions" | grep -q "${words[i]}"; then
                            region=${words[i]}
                        fi
                    done
                    local -r svcs="$(shipcat list-services "$region")"
                    COMPREPLY=($(compgen -W "$svcs" -- "$cur"))
                fi
                ;;
            cluster)
                local clustr_sub i
                for (( i=2; i < ${#words[@]}-1; i++ )); do
                    if [[ ${words[i]} = @(helm) ]]; then
                        clustr_sub=${words[i]}
                    fi
                done

                if [[ $prev = "cluster" ]]; then
                    COMPREPLY=($(compgen -W "helm" -- "$cur"))
                elif [[ $clustr_sub = "helm" ]]; then
                    # Suggest subcommands of helm and global flags
                    COMPREPLY=($(compgen -W "diff reconcile install" -- "$cur"))
                fi
                ;;
            helm)
                local -r regions="$(shipcat list-regions)"
                local helm_sub i
                for (( i=2; i < ${#words[@]}-1; i++ )); do
                    if [[ ${words[i]} = @(values|template|diff|upgrade|install) ]]; then
                        helm_sub=${words[i]}
                    fi
                done

                if [[ $prev = "helm" ]]; then
                    local -r region=$(kubectl config current-context)
                    local -r svcs="$(shipcat list-services "$region")"
                    COMPREPLY=($(compgen -W "$svcs" -- "$cur"))
                elif [ -n "${helm_sub}" ]; then
                    # TODO; helm sub command specific flags here
                    COMPREPLY=($(compgen -W "-o --output --dry-run" -- "$cur"))
                else
                    # Suggest subcommands of helm and global flags
                    COMPREPLY=($(compgen -W "values template diff upgrade install recreate" -- "$cur"))
                fi
                ;;
            jenkins)
                local -r regions="$(shipcat list-regions)"
                local jenk_sub i
                for (( i=2; i < ${#words[@]}-1; i++ )); do
                    if [[ ${words[i]} = @(latest) ]]; then
                        jenk_sub=${words[i]}
                    fi
                done

                if [[ $prev = "jenkins" ]]; then
                    local -r region=$(kubectl config current-context)
                    local -r svcs="$(shipcat list-services "$region")"
                    COMPREPLY=($(compgen -W "$svcs" -- "$cur"))
                elif [ -n "${jenk_sub}" ]; then
                    # TODO; jenkins sub command specific flags here
                    COMPREPLY=($(compgen -W "-o --output --dry-run" -- "$cur"))
                else
                    # Suggest subcommands of helm and global flags
                    COMPREPLY=($(compgen -W "latest console history" -- "$cur"))
                fi
                ;;
            shell|logs)
                svcs=$(find "./services" -maxdepth 1 -mindepth 1 -type d -printf "%f " 2> /dev/null)
                if [[ $prev = @(shell|logs) ]]; then
                    COMPREPLY=($(compgen -W "-r --region -p --pod $svcs" -- "$cur"))
                elif [[ $prev == @(-r|--region) ]]; then
                    local -r regions="$(shipcat list-regions)"
                    COMPREPLY=($(compgen -W "$regions" -- "$cur"))
                elif [[ $prev == @(-p|--pod) ]]; then
                    local -r pods="1 2 3 4 5 6"
                    COMPREPLY=($(compgen -W "$pods" -- "$cur"))
                else
                    COMPREPLY=($(compgen -W "$svcs" -- "$cur"))
                fi
                ;;
        esac
    fi

    return 0
} &&
complete -F _shipcat shipcat
