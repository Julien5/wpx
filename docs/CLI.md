# WPX command line

With the default arguments on `alpes.gpx`, this results in [this PDF](./sample/alpes.pdf).

```
wpx data/ref/alpes.gpx --output pdf/alpes.pdf
```
![](./images/alpes-small.png).
```
$ wpx  --help
Reads a GPX files and generates a PDF feuille de route and cutoff points

Usage: wpx [OPTIONS] [gpx]...

Arguments:
  [gpx]...  

Options:
      --segment-length <segment_length>
          the segment length in kilometer [default: 110]
      --segment-overlap <segment_overlap>
          the segment overlap in kilometer [default: 10]
      --start-time <start_time>
          start date time in ISO 8601 format, like 2026-01-10T20:00 [default: now]
      --speed <speed>
          speed in kilometer per hour [default: 15]
      --step-distance <step_distance>
          generate one cutoff point every [distance] kilometer [default: 10]
      --step-elevation-gain <step_elevation_gain>
          generate one cutoff point every [evelation gain] meter
      --kinds <kinds>
          [default: Controls GPXWaypoints Cities Mountains Villages Hamlets CutOff] [possible values: Cities, Controls, GPXWaypoints, Hamlets, Mountains, Villages, CutOff]
      --output <ouput>
          filename for the ouput (zip or pdf)
  -h, --help
          Print help

```
