# onesource demo
# Run with: vhs demo.tape

Output demo.gif

Set FontSize 14
Set Width 900
Set Height 500
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 60ms

# --- Scene 1: Just run it ---
# Show the simplest case — no flags, just works

Sleep 1s
Type "# Pack the whole project into one file"
Sleep 800ms
Enter
Sleep 300ms

Type "onesource"
Sleep 400ms
Enter
Sleep 3s

# --- Scene 2: Filter by file type ---
# Show that you can target just what you need

Sleep 1s
Type "# Only want the Rust source files?"
Sleep 800ms
Enter
Sleep 300ms

Type `onesource -i "*.rs" -o rust_only.txt`
Sleep 400ms
Enter
Sleep 3s

# --- Scene 3: Dry run before you commit ---
# Show the safety net

Sleep 1s
Type "# Not sure what will be packed? Preview first"
Sleep 800ms
Enter
Sleep 300ms

Type "onesource --dry-run"
Sleep 400ms
Enter
Sleep 3s

# --- Scene 4: Save your settings ---
# Show it's a real tool with memory

Sleep 1s
Type `# Save settings so next time you just type: onesource`
Sleep 800ms
Enter
Sleep 300ms

Type `onesource -i "*.rs" -x "target/" --save`
Sleep 400ms
Enter
Sleep 2s

Type "onesource"
Sleep 400ms
Enter
Sleep 3s

Sleep 2s
