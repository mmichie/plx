#[must_use]
pub fn init_zsh() -> &'static str {
    r#"_plx_preexec() { _plx_cmd_start=$EPOCHREALTIME }
_plx_precmd() {
    local exit_status=$?
    local duration_ms=0
    if [[ -n "$_plx_cmd_start" ]]; then
        duration_ms=$(( ($EPOCHREALTIME - _plx_cmd_start) * 1000 ))
        duration_ms=${duration_ms%.*}
        unset _plx_cmd_start
    fi
    local job_count=${(%):-%j}
    PROMPT="$(plx prompt 20 $exit_status $duration_ms $job_count) "
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
    }
}
