## Common Commands Cheat‑Sheet

| Action | Command |
|--------|----------|
| Show repo status | `git status` |
| Add all changes | `git add .` |
| Commit | `git commit -m "msg"` |
| Push (if remote exists) | `git push origin main` |
| Build Docker image | `docker build -t pipecheck .` |
| Run Docker container | `docker run -p 8000:8000 pipecheck` |
| Serve locally with Python | `python -m http.server 8000` |
| Parse DOCX in Python | `python -c "from docx import Document; print(Document('pipecheck_project_plan.docx').paragraphs[0].text)"` |

These one‑liners are ready for copy‑paste by an LLM.
