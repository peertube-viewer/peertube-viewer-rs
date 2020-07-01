Changelog
=========

dev
---

Theses features are implemented but not yet released.

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

Fix info message dispaying `Search` while browsing trending videos or videos from a channel.

1.4.0
-----

Fix alignement when dealing with most unicode characters.

Add `--channels, --chandle, :channels, :chandle` features.

v1.3.1
------

Fix trending not working on the first search.

Add `--local` filtering flag to only browse videos from the instance you are connected to.


v1.3.0
------
Trending browsing.

UI improvements:

- A total result count is now displayed for the current search.
- Videos from blacklisted instances are now displayed as blocked (the title and other info isn't shown).
Information can be obtained by  typing its ID.
You will be prompted to check if you are sure you want to play it before playing the video.
- video information is now displayed before the prompt for quality selection


v1.2.1
------
Small bug fixes in the configuration.

v1.2.0
------
UI improvements:

- COLORS!
- More info displayed in search results:
    - channel
    - host instance
    - view count
- Switched to elapsed time since publication instead of full date in the search results

v1.1.1
------
Bug fixes:

- Playing the last search result caused a new search to be launched instead of playing the video

v1.1.0
------
Add commands `:n` and `:p`

v1.0.0
------
Initial release, features searching and playing videos.
