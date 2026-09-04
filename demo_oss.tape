# VHS Tape for Tokenectomy OSS Demo
# Generates demo.gif for GitHub README

Output demo.gif

Set FontSize 14
Set Width 1050
Set Height 720
Set Padding 20
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set TypingSpeed 45ms

# Prepare environment invisibly
Hide
Type "cd /home/nans/Tokenonmix/Tokenectomy-OSS && export PATH=/home/nans/Tokenonmix/Tokenectomy-OSS/target/release:$PATH && ./setup_demo.sh && clear"
Enter
Sleep 600ms
Show

# 1. Unfiltered Crash Dump (The Pain)
Type "python3 server.py"
Enter
Sleep 2.5s

# 2. Pipe to Tokenectomy (The Solution)
Type "python3 server.py 2>&1 | tokenectomy"
Enter
Sleep 3.5s

# 3. Repeat to demonstrate Local SHA-256 Caching ($0 cost)
Type "python3 server.py 2>&1 | tokenectomy"
Enter
Sleep 3.5s
