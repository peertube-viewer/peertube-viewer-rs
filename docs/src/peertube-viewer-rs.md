peertube-viewer-rs
==================

**peertube-viewer-rs** is a small piece of software inspired by [youtube-viewer](https://github.com/trizen/youtube-viewer) made to allow quick browsing of videos available on [peertube](https://joinpeertube.org).

Both have similar interfaces.

peertube-viewer and peertube-viewer-rs
--------------------------------------

**peertube-viewer-rs** is a rewrite in rust of [peertube-viewer](https://gitlab.com/SostheneGuedon/peertube-viewer).
Though the implementation language doesn't matter for the end user, the name has been changed since breaking changes have been introduced, mainly in the configuration.

Usage
-----

Here is a screenshot of a basic usage of **peertube-viewer-rs**.
When launching  you are prompted with a search input.
Once a search is done, you can either select one of the results by writing the corresponding number or do another search by typing anything else.
When you chose to play a video, it will be launched in a video player ([mpv](https://github.com/mpv-player/mpv) by default)

![peertube-viewer-rs interface](screenshots/screenshot.png)
