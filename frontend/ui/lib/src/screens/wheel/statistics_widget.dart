import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class StatisticsWidget extends StatefulWidget {
  const StatisticsWidget({super.key});

  @override
  State<StatisticsWidget> createState() => _StatisticsWidgetState();
}

class _StatisticsWidgetState extends State<StatisticsWidget> {
  @override
  Widget build(BuildContext ctx) {
    RootModel rootModel = Provider.of<RootModel>(ctx);
    bridge.SegmentStatistics statistics = rootModel.statistics();
    double km = statistics.distanceEnd / 1000;
    double hm = statistics.elevationGain;
    return Container(
      constraints: const BoxConstraints(maxWidth: 500), // Set max width
      child: DataTable(
        columns: const [
          DataColumn(label: Text("distance"), numeric: true),
          DataColumn(label: Text("elevation"), numeric: true),
        ],
        rows: [
          DataRow(
            cells: [
              DataCell(Text("${km.toStringAsFixed(0)} km")),
              DataCell(Text("${hm.toStringAsFixed(0)} m")),
            ],
          ),
        ],
      ),
    );
  }
}
