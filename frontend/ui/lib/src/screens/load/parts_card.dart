import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/widgets/small.dart';
import 'model.dart';

class PartsCard extends StatelessWidget {
  const PartsCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    if (!model.hasDone(Job.parts)) {
      return Card(elevation: 4, child: Text("loading ..."));
    }
    List<bridge.TrackPart> parts = model.parts();
    List<TableRow> rows = [
      TableRow(children: [SmallText(text: "Segments"), SmallText(text: "")]),
    ];
    for (bridge.TrackPart part in parts) {
      rows.add(
        TableRow(children: [SmallText(text: ""), SmallText(text: part.name)]),
      );
    }

    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: rows,
    );

    return Card(elevation: 4, child: inner);
  }
}
