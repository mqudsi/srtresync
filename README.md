#srtresync 1.0

`srtresync` is a simple, straight-forward, and powerful utility used to correct ("resync") drift in srt subtitle files.

`srtresync` is developed and maintained by Mahmoud Al-Qudsi \<mqudsi@neosmart.net\> on behalf of [NeoSmart Technologies](https://neosmart.net/), and is released completely open source under the terms of the MIT license. See the `LICENSE` file for further information.


##Modes of Operation

Unlike most other utilities, `srtresync` can be used to correct both a fixed offset (say subtitles are consistently five and a half seconds behind) or linear drift (the difference between the subtitles and the audible dialog changes at a fixed rate, so that the difference/delay between audio and subtitles at the start of a film or clip is different than at the end).

###Correcting Fixed Offset

In its simplest mode of operation, `srtresync` can be used to add or subtract a constant amount from the current display times for a `.srt` subtitle file. Usage in this mode is exceedingly straightforward:

    srtresync ./input.srt [+/-]hh:mm:ss.xxx
    
This will lead `srtresync` to read the current subtitles from `./input.srt` and to write the output to the console, adjust forwards or backwards by hh:mm:ss.xxx hours/minutes/seconds/milliseconds (the +/- can be omitted if adding). 

###Correcting Linear Drift

A common anomaly is to have a subtitle file which starts off either in-sync with the spoken audio track or deviating from it by only a fixed offset, but then to have the difference between the two either increase or decrease at a constant pace, such that by the end of the clip the difference is no longer whatever it started out as.

`srtresync` can handle this quite easily, requiring you to only identify two differences from which it will derive the data needed to calculate and adjust for linear drift. For best results, a difference from the beginning of the clip/film and one from towards the very should be used (to maximize precision):

    srtresync ./input.srt hh:mm:ss.xxx-hh:mm:ss.xxx hh:mm:ss.xxx-hh:mm:ss.xxx

For example, if you have a clip and corresponding srt where the subtitles currently displaying at 00:00:10.500 should actually be shown at 00:00:07.200 and the subtitles showing at 02:14:03.007 should actually be shown at 02:22:17.400, then you would execute `srtresync` as follows:

    srtresync ./input.srt 00:00:10.500-00:00:07.200 02:14:03.007-02:22:17.400
    
`srtresync` will automatically calculate the linear drift parameters and adjust the output accordingly. In addition, it'll print the calculated variables to `stderr` for use as a quick sanity check:

> Offset will be 00:00:03,950 scaled by time at a rate of -0.06195986

Indicating that the initial offset was determined to be 3.950 seconds, and that as the movie goes on, this value will adjust with a linear coefficient of approximately -0.062 with regards to the time (in milliseconds). This makes sense, because we start off with with the subtitles 3.3 seconds _behind_ and end with the subtitles being 494.4 seconds _ahead_ so our positive initial offset needs to shrink towards zero and then beyond as the film progresses.