# 🎉 PipeChecker v0.2.8 - NPM Package Fix

## ✨ What's New

### NPM Package Binary Path Fix

We've fixed a critical bug in the npm package that prevented users from running PipeChecker immediately after installation.

---

## 🐛 Fix

### Problem
The npm wrapper script was looking for a **folder** instead of the **actual binary file** inside:

| Before (Broken) | After (Fixed) |
|-----------------|---------------|
| `npm/pipechecker-linux-x64` | `npm/pipechecker-linux-x64/pipechecker` |

This caused users to see an error when running `pipechecker` after `npm install pipechecker`.

---

### Solution
Updated `bin/pipechecker.js` to append `/pipechecker` suffix to the binary path:

```javascript
// Before (wrong)
return `pipechecker-${platform}-${arch}`;

// After (correct)
return `pipechecker-${platform}-${arch}/pipechecker`;
```

Also fixed for Windows:
```javascript
return `pipechecker-${arch}.exe/pipechecker.exe`;
```

---

## 📦 For NPM Users

### Install
```bash
npm install -g pipechecker
```

### Use
```bash
pipechecker .github/workflows/ci.yml
```

**Now works "straight out of the box" for everyone!** 🎉

---

## 📝 Changelog

### v0.2.8
- ✅ Fixed npm wrapper binary path
- ✅ Users can run pipechecker immediately after npm install
- ✅ Works on Linux, macOS, and Windows

### v0.2.7
- ✅ CircleCI global env vars now parsed
- ✅ CircleCI service images now parsed

---

**PipeChecker v0.2.8 — Fixed for npm users! 🚀**