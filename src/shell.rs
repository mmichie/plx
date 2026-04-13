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

#[cfg(test)]
mod tests {
    use super::init_zsh;

    #[test]
    fn contains_hooks() {
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
}
