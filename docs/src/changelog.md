Changelog
=========

1.8.1
---

- Update all dependencies
- Fix some warnings
- Reduce the size of the binary by removing a dependency

1.8.0
---

- Add support for using sepia search
- Add support for livestreams
- Add support for giving a link to a channel in the cli args

- Fix some bugs

1.7.3
---

- Support for HLS videos with the --use-raw-url option

1.7.2
---

- User agent configuration
- Fix multiple panics
- Avoids being stuck in selecting a resolution with no option when no option is available

1.7.1
-----

- add `:help` menu

1.7.0
-----

- Syntax highlighting in the prompt
- Tab completion in the prompt

- Halved compile times
- Halved binary size

1.6.3
-----

- Some internal changes
- Better error messages

1.6.2
-----

Replace white/blacklist by allow/blocklist.

1.6.1
-----

Add command `:browser` to open items in the browser.

1.6.0
---

Add comments support via the `:comments` command.

1.5.2
---

Add `edit-mode` config option.

1.5.1
---

Migrate the project to a dedicated group on gitlab.

1.5.0
---

Add `:info` to get info on a specific item.

1.4.1
-----

Fix info message displaying `Search` while browsing trending videos or videos from a channel.

1.4.0
-----

Fix alignment when dealing with most unicode characters.

Add `--channels, --chandle, :channels, :chandle` features.

1.3.1
------

Fix trending not working on the first search.

Add `--local` filtering flag to only browse videos from the instance you are connected to.


1.3.0
------
Trending browsing.

UI improvements:

- A total result count is now displayed for the current search.
- Videos from blacklisted instances are now displayed as blocked (the title and other info isn't shown).
Information can be obtained by  typing its ID.
You will be prompted to check if you are sure you want to play it before playing the video.
- video information is now displayed before the prompt for quality selection


1.2.1
------
Small bug fixes in the configuration.

1.2.0
------
UI improvements:

- COLORS!
- More info displayed in search results:
    - channel
    - host instance
    - view count
- Switched to elapsed time since publication instead of full date in the search results

1.1.1
------
Bug fixes:

- Playing the last search result caused a new search to be launched instead of playing the video

1.1.0
------
Add commands `:n` and `:p`

1.0.0
------
Initial release, features searching and playing videos.
