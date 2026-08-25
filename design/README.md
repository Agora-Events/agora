# Agora Design Contributions

This directory is specifically created for Open Source design contributions.

If you are tackling a design issue (e.g. `[Design] Signed-In User Home Screen`), you must:

1. Create a markdown file here named after your issue (e.g. `issue-1.md`).
2. Include the link to your Figma design.
3. Push your branch and open a Pull Request against `main`.
4. Include a screenshot of your design in your Pull Request description.

_For Figma access, contact `@divine_oseh` on Telegram._

## Pages Overview

| Route | Status | Description |
| :--- | :--- | :--- |
| `/` | **Live** | Landing page with hero section and featured events. |
| `/home` | **Live** | Main dashboard for authenticated users. |
| `/discover` | **Live** | Search and filter interface for finding events. |
| `/events/[id]` | **Live** | Detailed view of a specific event. |
| `/create-event` | **In-Progress** | Form to create and host new events. |
| `/profile` | **Mocked** | User profile settings and history. |
| `/auth` | **Live** | Login and Registration gateway. |

## How to Run

To get the development environment running locally:

### 1. Install Dependencies
Ensure you have [pnpm](https://pnpm.io/) installed globally.
```bash
pnpm install