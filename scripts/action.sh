# 用法：source scripts/action.sh
# action [路径] [--zed | --code] [--host SSH别名]
action() {
    local editor= host= path= positional=0 byte hex uri= i
    while (($#)); do
        if ((positional)); then
            [[ -z $path ]] || { echo '只能指定一个路径' >&2; return 1; }
            path=$1; shift; continue
        fi
        case $1 in
            --code|--zed)
                [[ -z $editor ]] || { echo '只能指定一个编辑器' >&2; return 1; }
                editor=${1#--} ;;
            --host)
                [[ -z $host && -n ${2:-} && $2 != -* ]] || { echo '请指定 SSH 主机别名' >&2; return 1; }
                host=$2; shift ;;
            --) positional=1 ;;
            -*) echo "未知选项：$1" >&2; return 1 ;;
            *) [[ -z $path ]] || { echo '只能指定一个路径' >&2; return 1; }; path=$1 ;;
        esac
        shift
    done
    editor=${editor:-code}; path=${path:-.}
    if [[ -z $host ]]; then command "$editor" -- "$path"; return; fi
    [[ $host =~ ^[a-zA-Z0-9._-]+$ && $path == /* ]] || {
        echo '远程主机须为 SSH 别名，路径须为绝对路径' >&2; return 1
    }
    local LC_ALL=C
    for ((i = 0; i < ${#path}; i++)); do
        byte=${path:i:1}
        case $byte in
            [a-zA-Z0-9.~_/-]) uri+=$byte ;;
            *) printf -v hex '%d' "'$byte"; printf -v hex '%%%02X' "$((hex & 255))"; uri+=$hex ;;
        esac
    done
    if [[ $editor == zed ]]; then
        command zed "ssh://$host$uri"
    else
        command code --folder-uri "vscode-remote://ssh-remote+$host$uri"
    fi
}
