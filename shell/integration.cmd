@echo off
REM Nudge - CMD Integration
REM Run this script or add to your AutoRun registry key
REM
REM Note: CMD has limited interactive capabilities compared to PowerShell.
REM This script provides a basic 'nudge-complete' command that you can use
REM to get suggestions for a given command prefix.
REM
REM Usage:
REM   nudge-complete git sta     -> suggests completion for "git sta"
REM   nudge-suggest              -> alias for nudge-complete
REM
REM To make this permanent, add to your AutoRun registry:
REM   reg add "HKCU\Software\Microsoft\Command Processor" /v AutoRun /t REG_SZ /d "path\to\integration.cmd" /f

REM Ensure daemon is running on first load
REM Only show message in interactive sessions to avoid breaking scp, rsync, etc.
if not defined NUDGE_LOADED (
    set NUDGE_LOADED=1
    REM Check if daemon is already running before starting
    nudge status >nul 2>&1
    if errorlevel 1 (
        start /b nudge start >nul 2>&1
    )
    REM Check if running interactively: PROMPT is set in interactive CMD sessions
    if defined PROMPT (
        echo Nudge loaded. Use 'nudge-complete ^<command^>' to get suggestions.
    )
)

REM Create doskey macros
doskey nudge-complete=nudge complete --format plain --buffer $* --cursor 9999 --cwd "%CD%" --session "cmd-%RANDOM%" --shell-mode "cmd-inline" 2^>nul
doskey nudge-suggest=nudge complete --format plain --buffer $* --cursor 9999 --cwd "%CD%" --session "cmd-%RANDOM%" --shell-mode "cmd-inline" 2^>nul

REM Alias for starting/stopping daemon
doskey nudge-start=nudge start
doskey nudge-stop=nudge stop
doskey nudge-status=nudge status
