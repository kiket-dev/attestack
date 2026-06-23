# Record a quickstart terminal demo → site/src/assets/img/quickstart-demo.gif
# Requires: vhs (https://github.com/charmbracelet/vhs), attestack on PATH

Output site/src/assets/img/quickstart-demo.gif
Set Width 920
Set Height 480
Set FontSize 15
Set Padding 20
Set Margin 12
Set Theme "Catppuccin Mocha"
Set Shell bash
Set TypingSpeed 35ms

Hide
Type "rm -rf /tmp/atvhs && mkdir -p /tmp/atvhs && cd /tmp/atvhs"
Enter
Type "git init -qb main && git config user.email demo@attestack.local && git config user.name Demo"
Enter
Type "echo hello > README.md && git add README.md && git commit -qm init"
Enter
Sleep 400ms
Show

Type "# Attestack quickstart"
Enter
Sleep 400ms
Type "attestack init"
Enter
Sleep 700ms
Type 'attestack start "feature work"'
Enter
Sleep 700ms
Type "attestack run -- echo tests-passed"
Enter
Sleep 600ms
Type 'attestack note "Reviewed auth path"'
Enter
Sleep 600ms
Type "attestack stop && attestack bundle create"
Enter
Sleep 900ms
Type "attestack verify .attestack/bundles/*.attestack.zip"
Enter
Sleep 1200ms
