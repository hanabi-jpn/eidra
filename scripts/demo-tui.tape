# Eidra TUI Dashboard Demo
Output docs/demo.gif

Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 10

# First show scan capability
Type "# Eidra — See what your AI tools are leaking"
Enter
Sleep 1s

Type "echo 'AKIAIOSFODNN7EXAMPLE password=hunter2 john@acme.com postgres://admin:secret@db.corp/prod -----BEGIN RSA PRIVATE KEY----- 090-1234-5678' | eidra scan"
Enter
Sleep 4s

Type ""
Enter
Type "# 47 rules. Zero config. Everything on-device."
Enter
Sleep 2s

Type ""
Enter
Type "eidra --help"
Enter
Sleep 3s
