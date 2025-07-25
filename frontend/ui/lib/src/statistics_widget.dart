import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class StatisticsWidget extends StatefulWidget {
  const StatisticsWidget({super.key});

  @override
  State<StatisticsWidget> createState() => _StatisticsWidgetState();
}

class _StatisticsWidgetState extends State<StatisticsWidget> {
  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        bridge.SegmentStatistics statistics = segmentsProvider.statistics();
        double km = statistics.distanceEnd / 1000;
        double hm = statistics.elevationGain;
        developer.log(
          "[SegmentsConsumer] length=${segmentsProvider.segments().length}",
        );
        return SizedBox(
          width: 500,
          child: DataTable(
            headingRowHeight: 0,
            columns: const [
              DataColumn(label: Text("")),
              DataColumn(label: Text(""), numeric: true),
            ],
            rows: [
              DataRow(
                cells: [
                  DataCell(Text("distance")),
                  DataCell(Text("${km.toStringAsFixed(0)} km")),
                ],
              ),
              DataRow(
                cells: [
                  DataCell(Text("elevation")),
                  DataCell(Text("${hm.toStringAsFixed(0)} m")),
                ],
              ),
            ],
          ),
        );
      },
    );
  }
}
