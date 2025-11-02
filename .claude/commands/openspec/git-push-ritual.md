---
name: Git Push Ritual
description: Complete git workflow: add, commit, and push changes to remote repository.
category: Git
tags: [git, push, commit, workflow]
---
<!-- OPENSPEC:START -->
**Guardrails**
- Always check git status before making changes to understand current state
- Review staged changes before committing to ensure only intended files are included
- Use descriptive commit messages that follow the project's commit message conventions
- Ensure working directory is clean before pushing

**Steps**
1. **Status Check**:
   - Run `git status` to review current changes and working directory state
   - Identify which files need to be added or committed
   - Check for any untracked files that should be handled

2. **Add Changes**:
   - Stage all relevant changes using `git add .` or specific files with `git add <file>`
   - Review what will be staged before proceeding
   - Ensure sensitive files or temporary changes are not accidentally staged

3. **Commit Changes**:
   - Create a descriptive commit message following project conventions
   - Use conventional commit format if project requires it (feat:, fix:, docs:, etc.)
   - Include relevant context about what was changed and why
   - Execute `git commit -m "<commit message>"`

4. **Push to Remote**:
   - Push changes to the appropriate remote branch
   - Use `git push` for current branch or `git push origin <branch>` for specific branch
   - Handle any push conflicts or authentication issues if they arise
   - Verify push was successful

5. **Verification**:
   - Confirm remote repository reflects the changes
   - Check CI/CD pipeline status if applicable
   - Verify working directory is clean after push

**Reference**
- Use `git diff --cached` to review staged changes before committing
- Use `git log --oneline -5` to review recent commit history
- Use `git remote -v` to verify remote repository configuration
- Consider using `git pull --rebase` before pushing if working with others
<!-- OPENSPEC:END -->