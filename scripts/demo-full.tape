# Eidra Full Demo — scan showcase
Output docs/demo.gif

Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 15
Set TypingSpeed 40ms

# Show scan in action
Type "echo '=== Eidra — See what your AI tools are leaking ==='"
Enter
Sleep 1s

Type `echo 'Deploy with AKIAIOSFODNN7EXAMPLE contact john@acme.com password="hunter2"' | eidra scan`
Enter
Sleep 3s

Type ""
Enter

Type `echo '-----BEGIN RSA PRIVATE KEY----- sk-ant-api1234567890abcdefghijklmnop redis://admin:pass@db.internal.corp/0 090-1234-5678' | eidra scan`
Enter
Sleep 3s

Type ""
Enter

Type `echo '=== MCP Semantic RBAC — blocks destructive AI actions ==='`
Enter
Sleep 1s

Type `echo 'AI tries: execute_sql("DROP TABLE users") → BLOCKED'`
Enter
Type `echo 'AI tries: read_file("/etc/shadow") → BLOCKED'`
Enter
Type `echo 'AI tries: run_command("rm -rf /") → BLOCKED'`
Enter
Type `echo 'AI tries: execute_sql("SELECT * FROM users") → ALLOWED'`
Enter
Sleep 2s

Type ""
Enter
Type "eidra --help"
Enter
Sleep 3s
