#[must_use]
pub fn init_zsh() -> &'static str {
    r#"_plx_preexec() {
    _plx_cmd_start=$EPOCHREALTIME
    _plx_cmd_title="$1"
}
_plx_precmd() {
    local exit_status=$?
    local duration_ms=0
    if [[ -n "$_plx_cmd_start" ]]; then
        duration_ms=$(( ($EPOCHREALTIME - _plx_cmd_start) * 1000 ))
        duration_ms=${duration_ms%.*}
        unset _plx_cmd_start
    fi
    local job_count=${(%):-%j}
    local plx_output
    plx_output="$(plx prompt 20 $exit_status $duration_ms $job_count)"
    PROMPT="${plx_output%%$'\n'*} "
    if [[ -n "$TMUX" && "$plx_output" == *$'\n'* ]]; then
        local tmux_title="${plx_output#*$'\n'}"
        local priority=$(tmux show-options -w -v @priority_title 2>/dev/null)
        if [[ -z "$priority" ]]; then
            tmux set-option -p @custom_title "" \; set-option -p @dir_title "$tmux_title" \; rename-window "$tmux_title"
        fi
    fi
    unset _plx_cmd_title
}
autoload -Uz add-zsh-hook
add-zsh-hook precmd _plx_precmd
add-zsh-hook preexec _plx_preexec
"#
}

#[must_use]
pub fn init_bash() -> &'static str {
    r#"_plx_preexec() {
    [[ -n "$_plx_in_precmd" ]] && return
    _plx_cmd_start=${_plx_cmd_start:-$EPOCHREALTIME}
}
_plx_precmd() {
    local exit_status=$?
    _plx_in_precmd=1
    local duration_ms=0
    if [[ -n "$_plx_cmd_start" ]]; then
        duration_ms=$(LC_ALL=C awk "BEGIN { printf \"%d\", ($EPOCHREALTIME - $_plx_cmd_start) * 1000 }")
        unset _plx_cmd_start
    fi
    local job_count=$(( $(jobs -p 2>/dev/null | wc -l) ))
    local plx_output
    plx_output="$(PLX_SHELL=bash plx prompt 20 $exit_status $duration_ms $job_count)"
    PS1="${plx_output%%$'\n'*} "
    if [[ -n "$TMUX" && "$plx_output" == *$'\n'* ]]; then
        local tmux_title="${plx_output#*$'\n'}"
        local priority=$(tmux show-options -w -v @priority_title 2>/dev/null)
        if [[ -z "$priority" ]]; then
            tmux set-option -p @custom_title "" \; set-option -p @dir_title "$tmux_title" \; rename-window "$tmux_title"
        fi
    fi
    unset _plx_in_precmd
}
trap '_plx_preexec' DEBUG
PROMPT_COMMAND=_plx_precmd
"#
}

#[must_use]
pub fn init_fish() -> &'static str {
    r#"function fish_prompt
    set -l exit_status $status
    set -l duration_ms $CMD_DURATION
    set -l job_count (count (jobs -p 2>/dev/null))
    set -l plx_output (PLX_SHELL=fish command plx prompt 20 $exit_status $duration_ms $job_count)
    set -l lines (string split \n -- $plx_output)
    echo -n "$lines[1] "
    if set -q TMUX; and test (count $lines) -gt 1
        set -l priority (tmux show-options -w -v @priority_title 2>/dev/null)
        if test -z "$priority"
            tmux set-option -p @custom_title "" \; set-option -p @dir_title "$lines[2]" \; rename-window "$lines[2]"
        end
    end
end
"#
}

#[cfg(test)]
mod tests {
    use super::{init_bash, init_fish, init_zsh};

    #[test]
    fn zsh_contains_hooks() {
        let out = init_zsh();
        assert!(out.contains("add-zsh-hook precmd _plx_precmd"));
        assert!(out.contains("add-zsh-hook preexec _plx_preexec"));
        assert!(out.contains("EPOCHREALTIME"));
        assert!(out.contains("plx prompt"));
        assert!(
            out.contains("@priority_title"),
            "should check priority title"
        );
        assert!(out.contains("rename-window"), "should rename tmux window");
    }

    #[test]
    fn bash_contains_prompt_command() {
        let out = init_bash();
        assert!(
            out.contains("PROMPT_COMMAND=_plx_precmd"),
            "expected PROMPT_COMMAND"
        );
        assert!(
            out.contains("trap '_plx_preexec' DEBUG"),
            "expected DEBUG trap"
        );
        assert!(out.contains("PLX_SHELL=bash"), "expected PLX_SHELL=bash");
        assert!(out.contains("plx prompt"), "expected plx prompt call");
        assert!(out.contains("EPOCHREALTIME"), "expected EPOCHREALTIME");
        assert!(out.contains("rename-window"), "should rename tmux window");
    }

    #[test]
    fn bash_guards_against_precmd_reentry() {
        let out = init_bash();
        assert!(
            out.contains("_plx_in_precmd"),
            "expected reentry guard in bash init"
        );
    }

    #[test]
    fn fish_contains_fish_prompt() {
        let out = init_fish();
        assert!(
            out.contains("function fish_prompt"),
            "expected fish_prompt function"
        );
        assert!(out.contains("PLX_SHELL=fish"), "expected PLX_SHELL=fish");
        assert!(
            out.contains("CMD_DURATION"),
            "expected CMD_DURATION for timing"
        );
        assert!(out.contains("plx prompt"), "expected plx prompt call");
        assert!(out.contains("rename-window"), "should rename tmux window");
    }
}
