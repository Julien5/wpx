import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class WayPointsView extends StatefulWidget {
  final SegmentsProvider? segmentsProvider;
  final bridge.Segment? segment;
  const WayPointsView({super.key, this.segmentsProvider, this.segment});

  @override
  State<WayPointsView> createState() => WayPointsViewState();
}

class WayPointsViewState extends State<WayPointsView> {
  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    var local=widget.segmentsProvider!.waypointTable(widget.segment!);

    developer.log("[WayPointsViewState] [build] #_waypoints=${local.length}");

    if (local.isEmpty) {
      return const Center(child: Text("No waypoints available"));
    }

    return DataTable(
      columnSpacing: 20,
      dataRowMinHeight: 25,
      dataRowMaxHeight: 25,
      headingRowHeight: 30,
      columns: const [
        DataColumn(label: Text('KM'), numeric: true),
        DataColumn(label: Text('Time'), numeric: true),
        DataColumn(label: Text('Dist.'), numeric: true),
        DataColumn(label: Text('Elev.'), numeric: true),
        DataColumn(label: Text('Slope'), numeric: true),
      ],
      rows:
          local.asMap().entries.map((entry) {
            var index=entry.key;
            var info = entry.value.info!;
            var dt = DateTime.parse(info.time);
            var km = info.distance / 1000;
            var ikm = info.interDistance / 1000;
            var egain = info.interElevationGain;
            var slope = 100 * info.interSlope;

            return DataRow(
              color: WidgetStateProperty.resolveWith<Color?>((
                Set<WidgetState> states,
              ) {
                // Alternate row colors
                return index.isEven
                    ? const Color.fromARGB(255, 214, 201, 201)
                    : Colors.white;
              }),
              cells: [
                // ignore: unnecessary_string_interpolations
                DataCell(Text("${km.toStringAsFixed(0)}")),
                DataCell(Text(DateFormat('HH:mm').format(dt))),
                DataCell(Text("${ikm.toStringAsFixed(1)} km")),
                DataCell(Text("${egain.toStringAsFixed(0)} m")),
                DataCell(Text("${slope.toStringAsFixed(1)} %")),
              ],
            );
          }).toList(),
    );
  }
}

class WayPointsConsumer extends StatelessWidget {
  const WayPointsConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        var wp = Provider.of<ProfileRenderer>(context, listen: false);
        var segment = wp.segment;
        return Center(
          child: WayPointsView(
            segmentsProvider: segmentsProvider,
            segment: segment,
          ),
        );
      },
    );
  }
}
