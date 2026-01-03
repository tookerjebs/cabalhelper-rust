# Fixing Cargo Terminal Output in PowerShell

## Problem
Cargo build errors appear truncated or garbled in PowerShell.

## Quick Fix (This Session Only)
```powershell
$env:CARGO_TERM_COLOR = "never"
```

Then run cargo commands normally - output will be readable!

## Permanent Fix

### Method 1: Add to PowerShell Profile
```powershell
# Open your profile
notepad $PROFILE

# Add this line:
$env:CARGO_TERM_COLOR = "never"

# Save and restart PowerShell
```

### Method 2: Use Helper Aliases (Recommended)
```powershell
# Load once per session:
. C:\Users\Hello\.gemini\cargo_helpers.ps1

# Then use clean aliases:
ccheck              # instead of cargo check
cbuild              # instead of cargo build
cbuild -Release     # instead of cargo build --release
crun                # instead of cargo run
```

To make aliases permanent:
```powershell
Add-Content $PROFILE "`n. C:\Users\Hello\.gemini\cargo_helpers.ps1"
```

## Why This Works
Cargo outputs ANSI color codes that PowerShell can't decode properly. Disabling colors gives clean, readable output.

---

**That's it!** No more log files or redirection needed.
