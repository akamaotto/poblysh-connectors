---
name: Git Branch Ritual
description: Create and checkout a new branch from master/main using OpenSpec change name.
category: Git
tags: [git, branch, checkout, workflow]
---
<!-- OPENSPEC:START -->
**Guardrails**
- Always ensure working directory is clean before creating new branches
- Verify the target base branch (master/main) exists and is up to date
- Use descriptive branch names that clearly indicate the work being done
- Pull latest changes from base branch before creating new branch

**Steps**
1. **Preparation**:
   - Run `git status` to ensure working directory is clean
   - If there are uncommitted changes, either commit them or stash them
   - Identify the primary branch (master or main) by running `git branch -a`

2. **Update Base Branch**:
   - Switch to the primary branch: `git checkout master` or `git checkout main`
   - Pull latest changes: `git pull origin <branch-name>`
   - Verify you're on the correct branch with `git branch`

3. **Create New Branch**:
   - Use the OpenSpec change name provided as the branch name
   - Create and checkout the new branch: `git checkout -b <change-name>`
   - Verify the new branch was created successfully: `git branch`

4. **Verification**:
   - Confirm you're on the new branch: `git status`
   - Verify the branch tracks the correct base branch
   - Check that the branch name is appropriate and follows project conventions

5. **Ready for Work**:
   - Confirm the working directory is ready for development
   - Branch is now ready for implementing the OpenSpec change
   - Provide confirmation of successful branch creation

**Reference**
- Use `git stash` if you need to temporarily save uncommitted changes
- Use `git branch -a` to see all local and remote branches
- Use `git log --oneline -3` to verify you're starting from the latest commit
- Consider using `git fetch --all` to ensure you have the latest remote branch information
<!-- OPENSPEC:END -->