import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/desktop/kinds_row.dart';
import 'package:ui/src/screens/settings/settings_screen.dart';
import 'package:ui/src/screens/wheel/statistics_widget.dart';
import 'package:ui/src/utils/print.dart';
import 'package:ui/src/widgets/export.dart';
import 'package:ui/src/widgets/small.dart';
import 'package:ui/src/widgets/userstepsslider.dart';
import 'package:ui/src/utils/utils.dart';

class UserStepsCard extends StatefulWidget {
  final ExpansibleController controller;
  final void Function(BuildContext, ScreenFocus, bool) onExpansionChanged;
  const UserStepsCard({
    super.key,
    required this.controller,
    required this.onExpansionChanged,
  });

  @override
  State<UserStepsCard> createState() => _UserStepsCardState();
}

class _UserStepsCardState extends State<UserStepsCard> {
  @override
  Widget build(BuildContext context) {
    ParameterModel parameterModel = Provider.of<ParameterModel>(context);
    debugPrint("REBUILD PACING CARD");

    String pacingPointsText = getPacingPointText(parameterModel.parameters());
    Widget tile = ExpansionTile(
      title: Row(
        children: [
          SmallText(text: "Pacing points"),
          SmallText(text: pacingPointsText),
        ],
      ),
      controller: widget.controller,
      onExpansionChanged:
          (expanded) => widget.onExpansionChanged(
            context,
            ScreenFocus.usersteps,
            expanded,
          ),
      children: <Widget>[
        Builder(
          builder: (BuildContext context) {
            return UserStepsSliderProvider();
          },
        ),
      ],
    );
    return Card(elevation: 4, child: tile);
  }
}

class PDFCard extends StatefulWidget {
  final ExpansibleController controller;
  final void Function(BuildContext, ScreenFocus, bool) onExpansionChanged;
  const PDFCard({
    super.key,
    required this.controller,
    required this.onExpansionChanged,
  });

  @override
  State<PDFCard> createState() => _PDFCardState();
}

class _PDFCardState extends State<PDFCard> {
  @override
  Widget build(BuildContext context) {
    SegmentModel segment = Provider.of<SegmentModel>(context, listen: false);
    ParameterModel parameterModel = Provider.of<ParameterModel>(context);
    List<Segment> segments = segment.backend.segments();
    String segLength =
        (segmentLengthWithoutOverlap(parameterModel.parameters()) / 1000)
            .ceil()
            .toString()
            .padLeft(3);
    String pageCount = segments.length.toString().padLeft(2);

    Widget tile = ExpansionTile(
      title: Row(
        children: [
          SmallText(text: "PDF"),
          SmallText(text: "$pageCount pages, $segLength km per page"),
        ],
      ),
      controller: widget.controller,
      onExpansionChanged:
          (expanded) => widget.onExpansionChanged(
            context,
            ScreenFocus.settings,
            expanded,
          ),
      children: <Widget>[
        Row(
          children: [SmallText(text: "Number of pages:"), PagesSliderWidget()],
        ),
      ],
    );
    return Card(elevation: 4, child: tile);
  }
}

class SidePanel extends StatefulWidget {
  final double width;
  const SidePanel({super.key, required this.width});

  @override
  State<SidePanel> createState() => _SidePanelState();
}

class _SidePanelState extends State<SidePanel> {
  final ExpansibleController _userStepsController = ExpansibleController();
  final ExpansibleController _pdfController = ExpansibleController();

  void updateModelFromWidgets(
    BuildContext context,
    ScreenFocus f,
    bool expanded,
  ) {
    // do not change the state during build (otherwise exception)
    SchedulerBinding.instance.addPostFrameCallback((_) {
      debugPrint("focus:$f expanded:$expanded");
      FociModel fociModel = Provider.of<FociModel>(context, listen: false);
      if (expanded) {
        fociModel.addFocus(f);
      } else {
        fociModel.removeFocus(f);
      }
    });
  }

  void updateWidgetsFromModel(BuildContext context, FociModel fociModel) {
    if (fociModel.contains(ScreenFocus.usersteps)) {
      _userStepsController.expand();
      _pdfController.collapse();
    }

    if (fociModel.contains(ScreenFocus.settings)) {
      _userStepsController.collapse();
      _pdfController.expand();
    }
  }

  @override
  Widget build(BuildContext context) {
    FociModel fociModel = Provider.of<FociModel>(context);
    updateWidgetsFromModel(context, fociModel);
    Widget div = Divider(color: Colors.lightBlue, thickness: 1, height: 1);
    List<Widget> children = [
      div,
      Padding(
        padding: const EdgeInsets.all(15),
        child: Column(
          children: [
            OverviewWidget(
              onPacingPointPressed: null,
              onControlsPointPressed: () {},
              onPDFPressed: null,
            ),
            Card(elevation: 4, child: KindsRow()),
          ],
        ),
      ),
      div,
      Padding(
        padding: const EdgeInsets.all(15),
        child: UserStepsCard(
          controller: _userStepsController,
          onExpansionChanged: updateModelFromWidgets,
        ),
      ),
      div,
      Padding(
        padding: const EdgeInsets.all(15),
        child: PDFCard(
          controller: _pdfController,
          onExpansionChanged: updateModelFromWidgets,
        ),
      ),
      div,
      SizedBox(height: 20),
      Center(child: ExportButton(text: "export zip", type: Type.zip)),
    ];
    return ConstrainedBox(
      constraints: BoxConstraints(
        minWidth: widget.width,
        maxWidth: widget.width,
      ),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        children: children,
      ),
    );
  }
}
