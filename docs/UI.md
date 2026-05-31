# WPX frontend

![](./images/overview.png)

### Set Time Parameters

  - **Start Time:** Set the date and time. An accurate start date helps label the days correctly on the elevation profile for rides passing midnight.
  - **Speed:** The speed controls the cutoff time computation. It is set to 15.0 kmh by default. 
  - **End Time:** This can be adjusted to match the official brevet cutoff. This will recalculate the average speed accordingly.

*Note: The computed closing times for each control may not perfectly match the official organization times, but they should be very close.*

#### Speed/Cutoff times

The cutoff times are computed using the speed parameter. 
There are two options:

![](./images/speed-dialog.png)

- User-set overall average speed: this can be set to anything in the range 1-100kmh. 
- ACP rules: the option is available if the total distance is within a 50-km margin of a nominal brevet distance (200, 300, 400, 600, 1000 and 1200 kilometers). This applies the [ACP (Audax Club Parisien) rules](https://www.audax-club-parisien.com/wp-content/uploads/2024/01/Rules-for-rider-2024.pdf). 
- LRM rules: available if the total distance is longer than 1200 km. This applies the [LRM (Les Randonneurs Mondiaux) rules](https://www.randonneursmondiaux.org/files/Rules_2019.pdf).

#### ACP Rules 

The ACP rules are valid up to 1000 km, with an extension for the PBP.
They look simple at first sight ("15 kmh up to 600 km") but they are full of surprises. 
- The start point has a cutoff: one hour after the start time. 
- The speed in the range 1000-1200 km is 13.33 kmh, larger than in the range 600-1000 kmh. A rider cycling at "ACP speed" is expected to ride *faster* in the last 200 km of PBP.
- Because the *actual* distance of a brevet mostly differs from its nominal distance, the speed between the last control and the end of the ride is special. 
Example: Assuming a 630km long brevet (nominal 600km) with a last control at 570 km:
  - the last control closes after 570/15=38 hours, and
  - the end of the brevet must be reached after 40 hours.

   Hence, the last 60 kilometers must be covered in 2 hours, at 30kmh. A rider with a 1-hour time-on-hand at the last control would have 1+2=3 hours to cover the last 60 km. 

As you see, the ACP rules may not be appropriate to pace your ride. They may be rather interesting for comparison purposes. 

Here is a diagram that summarizes the ACP rule for a 600 km brevet with actual distance 630 km:
![](./images/ACP-600-diagram.png)

Here is a diagram that summarizes the ACP rule for a 1200 km brevet with actual distance 1230 km:
![](./images/ACP-diagram.png)

#### LRM Rules 

The LRM rules are valid for rides longer than 1200 km.
They are simpler than ACP rules and just define a fixed overall average speed.
- from 1200 to 1299 km: 13.33 kmh
- from 1300 to 1899 km: 12 kmh
- from 1900 to 2499 km: 10 kmh
- 2500 km and above: 200 km per day

In a 1200-km brevet with LRM rules, a control point at 600 km would close after 45h.

### Control Custom Cutoff

If the speed is a user-set overall average speed (first option), the control times can be set individually. 

Example on a PBP track: set the speed to 13.35 kmh and adjust the end time to get 90h duration. 
Then click on control time button:
![](./images/control-time-button.png)
This opens the control-time dialog, which allows you to modify the cutoff time: 
![](./images/control-time-dialog.png)

This is useful for longer brevet if you want a simpler and more predictable pacing than the ACP rules. In the example, the cutoff in Brest is set to 40h, the other controls are automatically spaced evenly. This is simpler and gets you to each control slightly before the official ACP cutoff:
![](./images/ACP-simple-diagram.png)

Of source, you may set your control times to fit your own personal strategy.

### Choose Point Types

  - **Waypoints:** Original waypoints from the input GPX. If a waypoint matches a control point, it is merged into the control (and not shown as a separate waypoint). Control points can be created from waypoints with the control checkbox in the overview list (only on desktop).
  - **OSM:** Names of cities, villages, and mountain passes. Only the most important OSM points are shown to keep the table readable (filtered by a minimum distance of 10% of the track/page length).
  - **Cutoff:** These are shown as dots on the elevation profile and map, but are excluded from the tables.

### PDF

  - Since two segments are printed on a single A4 page, you can select the number of pages in increments of 0.5 (e.g., 0.5, 1, 1.5, etc.).
  - WPX attempts to generate segments covering round distances (like 100km) with a 10% overlap. Due to these constraints, the slider does not offer every page count.

