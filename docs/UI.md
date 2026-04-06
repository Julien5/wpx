# UI

![](./images/screenshot-small.png)

### Set Time Parameters

  - **Start Time:** Set the date and time. An accurate start date helps label the days correctly on the elevation profile for rides passing midnight.
  - **Speed:** Currently, only constant speed is supported. This is generally sufficient for brevets up to 600km per ACP rules.
  - **End Time:** This can be adjusted to match the official brevet cutoff. This will recalculate the average speed accordingly.

*Note: The computed closing times for each control may not perfectly match the official organization times, but they should be very close.*

### Choose Point Types

  - **Controls:** Points found at the ends of segments.
  - **Waypoints:** Original waypoints from the input GPX. If a waypoint matches a control point, it is merged into the control (and not shown as a separate waypoint).
  - **OSM:** Names of cities, villages, and mountain passes. Only the most important OSM points are shown to keep the table readable (filtered by a minimum distance of 10% of the track/page length).
  - **Pacing:** These are shown as dots on the elevation profile and map, but are excluded from the tables.

### PDF

  - Since two segments are printed on a single A4 page, you can select the number of pages in increments of 0.5 (e.g., 0.5, 1, 1.5, etc.).
  - WPX attempts to generate segments covering round distances (like 100km) with a 10% overlap. Due to these constraints, the slider does not offer every page count.

