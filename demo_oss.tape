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

# 1. Unfiltered Crash Dump (The Problem)
Type "python3 server.py"
Enter
Sleep 2.5s

# 2. Pipe to Tokenectomy (The Solution)
Type "python3 server.py 2>&1 | tokenectomy"
Enter
Sleep 3.5s

# 3. Agent applies patch via Tokenectomy MCP Server
Type "cat mcp_call.json | tokenectomy --mcp"
Enter
Sleep 2.5s

# 4. Rerun server -> Problem Solved!
Type "python3 server.py"
Enter
Sleep 4.0s
