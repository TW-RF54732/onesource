# onesource profile demo
# Run from the project root with: vhs medias/simple_demo.tape

Output medias/simple_demo.gif

Set FontSize 14
Set Width 900
Set Height 500
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 60ms

# --- Scene 1: List saved profiles ---
Sleep 1s
Type "# Show saved profiles"
Sleep 800ms
Enter
Sleep 300ms

Type "onesource profile list"
Sleep 400ms
Enter
Sleep 2s

# --- Scene 2: Inspect a profile ---
Ctrl+L
Sleep 1s
Type `# Inspect the "core" profile settings`
Sleep 800ms
Enter
Sleep 300ms

Type "onesource profile show core"
Sleep 400ms
Enter
Sleep 2s

# --- Scene 3: Use a profile ---
Ctrl+L
Sleep 1s
Type `# Run onesource with the saved "core" profile`
Sleep 800ms
Enter
Sleep 300ms

Type "onesource -p core"
Sleep 400ms
Enter
Sleep 3s

# --- Scene 4: Explain path decisions ---
Ctrl+L
Sleep 1s
Type "# Explain why each path is included or skipped"
Sleep 800ms
Enter
Sleep 300ms

Type "onesource explain -p core README.md .git"
Sleep 400ms
Enter
Sleep 3s

Sleep 2s
