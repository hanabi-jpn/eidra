# Eidra Demo
Output docs/demo.gif

Set FontSize 14
Set Width 1200
Set Height 650
Set Theme "Catppuccin Mocha"
Set Padding 20

Type "# Eidra - See what your AI tools are leaking"
Enter
Sleep 1s

Type "echo 'AKIAIOSFODNN7EXAMPLE password=hunter2 john@acme.com postgres://admin:secret@db.internal.corp/prod' | eidra scan"
Enter
Sleep 3s

Type ""
Enter

Type "echo 'sk-ant-abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnop hvs.ABCDEFGHIJKLMNOPQRSTUVWXYZab' | eidra scan"
Enter
Sleep 3s

Type ""
Enter

Type "echo '-----BEGIN RSA PRIVATE KEY----- 090-1234-5678 redis://user:pass@cache.local/0' | eidra scan"
Enter
Sleep 3s

Type ""
Enter
Type "eidra --help"
Enter
Sleep 3s
