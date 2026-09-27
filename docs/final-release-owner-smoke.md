# Aether 0.5.0 final owner smoke checklist

Use the latest signed release candidate on a protected Windows test account with
non-personal test data, one valid MyTimetable feed, and one valid configured cloud
AI provider. Stop on any crash, secret exposure, authorization crossing, data loss,
approval bypass, route change, write-tool exposure, or false success/freshness state.

- [ ] 1. Start the latest `master` release candidate and confirm no obvious native or WebView errors.
- [ ] 2. Open **Settings → Connections**; confirm MyTimetable is available and Brightspace is absent.
- [ ] 3. Connect and refresh MyTimetable with the test feed; confirm validation, truthful sync status, and no displayed feed URL.
- [ ] 4. Open a School Space; associate the intended source and group.
- [ ] 5. Check **Today**, **Week**, and **Upcoming**, including timed/all-day/cancelled/overlapping test events.
- [ ] 6. If two test connections are available, verify identical group names remain isolated by connection and School Space.
- [ ] 7. Check Pulse **Now**, **Next**, **Today**, **Upcoming**, Tasks, conflicts, Continuity, and source trust labels.
- [ ] 8. Create, edit, complete, and reopen a Task; confirm due/overdue behavior and Pulse projection.
- [ ] 9. Create and edit a Note; confirm it persists and appears in relevant Search results.
- [ ] 10. Search across available domains; confirm labels are correct and removed/archived items do not leak.
- [ ] 11. Ask an ordinary cloud AI question; confirm normal streaming, persistence, and provider/model provenance.
- [ ] 12. Cancel an ordinary AI stream; confirm no stale completion or stuck activity remains.
- [ ] 13. Ask **“What is my next calendar event?”** from the authorized School Space.
- [ ] 14. Confirm visible tool activity and that disclosure approval appears before cloud continuation.
- [ ] 15. Confirm the approval names the selected provider/model and shows only a category/count summary; choose **Approve once**.
- [ ] 16. Confirm the calendar answer matches authorized local data and exposes no raw description, feed URL, configuration, or unrelated source.
- [ ] 17. Ask **“What tasks are due soon?”**; confirm bounded open Task results and no completed/archived or excluded full-body leakage.
- [ ] 18. Cancel a tool-enabled turn while it is active; confirm no continuation, extra tool call, false success, or stuck spinner.
- [ ] 19. Restart Aether; confirm Connections, School bindings, Tasks, Notes, AI history/provenance, and intended state persist.
- [ ] 20. Disable and re-enable MyTimetable; confirm cached data and trust/freshness labels remain truthful.
- [ ] 21. Test an unreachable/invalid feed and offline provider state; confirm sanitized errors, no crash, and no stuck syncing.
- [ ] 22. Disconnect/remove the test connection; confirm its School/Pulse data disappears without affecting unrelated connections.
- [ ] 23. Confirm Brightspace remains absent and no academic deadline is inferred from event text.
- [ ] 24. Confirm the AI registry offers no write/mutation tool and existing Safe Actions still require preview and approval.
- [ ] 25. Inspect Pulse, Settings/Connections, School, Tasks, Notes, Search, AI, and disclosure UI in light and dark modes.
- [ ] 26. Keyboard through key actions and the disclosure dialog; confirm visible focus, labelled controls, disabled-state clarity, Escape/Cancel behavior, and no color-only critical state.
- [ ] 27. Complete the signed installer/updater, upgrade, backup/restore, signature, tamper-rejection, and rollback checks in [the release runbook](release-runbook.md) before publication.

Record only privacy-minimal outcomes and artifact hashes. Do not copy credentials,
feed URLs, private content, paths, identifiers, databases, backups, or raw logs into
issues or release notes.
