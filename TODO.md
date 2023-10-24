# Work Warden TODO

What is the goal for this tool?
* Track when I start and stop work, like keeping a timecard, to keep me accountable
* Track time on specific projects and sync it to Shortcut
* Sync with Garmin Connect to see if I've worked out before work or not

How is this going to work?
* UI
    * Tauri window with different internal tabs:
        * Activity tracking page
            * Clock in/out buttons:
                * Day (start/stop)
                * Break (start/stop)
                * Lunch (start/stop)
            * Tasks list
                * Starred tasks first
                * Then most-recent tasks
                * Icon to indicate if synced with Shortcut
        * Settings page
            * Hours worked
            * Max idle working time
            * Max active non-working time
            * Workout length
        * History page
            * 3 views:
                * Daily
                * Weekly summary
                * Monthly summary
    * System tray icon
    * System notifications
* Backend
    * Track KDE idle state
    * Synchronize with Shortcut
    * Synchronize with Garmin Connect
    
Interactions:
* When system tray icon is clicked, main window is shown or hidden
* When notification is clicked, main window is shown
* When clocked out and switch from idle to active, show main window
* When not worked out, show warning in clock-in page, and show persistent notification
    * In clock-in page workout warning, add "remind me after" form that can ignore the notification for a period of time
    * Also add a "not today" button that requires 3 clicks
* On Tasks list, when task is clicked, start logging time towards it
    * If shift is held or checkbox on left is selected, then do multi-select. Time gets split evenly between tasks
* Backend saves log of day and can pick up where previous left off (in case of crash)

Rust modules:
* notifier
* timecard

Tasks:
[X] Timecard event log and status
[X] Timecard management
[X] UI with time tracking
[X] Track idle time
[X] Refresh timecards after day boundary
[ ] Notifications (overtime work, long lunch [until over], long break [until over])
[ ] Tasks list with starred and last-used metrics
[ ] Shortcut integration (API key, filters)
[ ] Settings page (Shortcut API key, force sync)
[ ] Garmin Connect integration (API key, settings)
