import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/screens/desktop/central_panel_overview.dart';
import 'package:ui/src/screens/desktop/central_panel_pdf.dart';
import 'package:ui/src/screens/desktop/central_panel_usersteps.dart';

class ProfilePadding extends StatelessWidget {
  final Widget child;

  const ProfilePadding({super.key, required this.child});
  @override
  Widget build(BuildContext ctx) {
    return Padding(
      padding: EdgeInsetsGeometry.fromLTRB(5, 10, 10, 10),
      child: child,
    );
  }
}

class GraphicsPadding extends StatelessWidget {
  final Widget child;

  const GraphicsPadding({super.key, required this.child});
  @override
  Widget build(BuildContext ctx) {
    return Padding(padding: EdgeInsetsGeometry.all(20), child: child);
  }
}

class CentralPanel extends StatefulWidget {
  final double width;
  const CentralPanel({super.key, required this.width});

  @override
  State<CentralPanel> createState() => _CentralPanelState();
}

class _CentralPanelState extends State<CentralPanel> {
  @override
  Widget build(BuildContext context) {
    FociModel fociModel = Provider.of<FociModel>(context);
    return IndexedStack(
      index:
          fociModel.contains(ScreenFocus.usersteps)
              ? 0
              : fociModel.contains(ScreenFocus.settings)
              ? 1
              : 2,
      children: [
        SizedBox(
          width: widget.width,
          child: CentralPanelUserSteps(width: widget.width),
        ),
        SizedBox(
          width: widget.width,
          child: CentralPanelPDF(width: widget.width),
        ),
        SizedBox(
          width: widget.width,
          child: CentralPanelOverview(width: widget.width),
        ),
      ],
    );
  }
}
