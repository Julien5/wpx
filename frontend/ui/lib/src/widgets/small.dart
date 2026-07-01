import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:url_launcher/url_launcher.dart';

class SmallButton extends StatelessWidget {
  final VoidCallback? callback;
  final String text;
  const SmallButton({super.key, this.callback, required this.text});

  @override
  Widget build(BuildContext context) {
    EdgeInsets valuePadding = const EdgeInsets.fromLTRB(15, 0, 15, 0);
    return Padding(
      padding: const EdgeInsets.all(8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.start,
        children: [
          OutlinedButton(
            onPressed: callback,
            style: ElevatedButton.styleFrom(
              padding: valuePadding,
              minimumSize: Size.zero,
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
            child: Text(text, textAlign: TextAlign.center),
          ),
        ],
      ),
    );
  }
}

class SmallInfoLink extends StatelessWidget {
  final String url;

  const SmallInfoLink({super.key, required this.url});

  // Function to handle launching the URL
  Future<void> _launchURL() async {
    final Uri uri = Uri.parse(url);

    if (!await launchUrl(
      uri,
      mode:
          LaunchMode
              .externalApplication, // Forces opening in a new browser tab/app
    )) {
      // throw Exception('Could not launch ${link.url}');
      debugPrint('Could not launch $url');
    }
  }

  @override
  Widget build(BuildContext context) {
    return RichText(
      text: TextSpan(
        style: const TextStyle(color: Colors.black, fontSize: 11),
        children: [
          WidgetSpan(
            alignment:
                PlaceholderAlignment
                    .middle, // perfectly aligns the icon vertically
            child: IconButton(
              icon: const Icon(Icons.info_outline_rounded),
              color: Colors.blue,
              iconSize: 15,
              padding: EdgeInsets.zero,
              constraints:
                  const BoxConstraints(), // Removes default extra padding around the icon
              tooltip:
                  'Open documentation', // Desktop hover text / Mobile long-press accessibility
              onPressed: _launchURL, // Handles both taps and mouse clicks
            ),
          ),
        ],
      ),
    );
  }
}

class SmallText extends StatelessWidget {
  final String text;
  const SmallText({super.key, required this.text});

  @override
  Widget build(BuildContext context) {
    context.watch<ScreenConfiguration>();
    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: Text(text, textAlign: TextAlign.left),
    );
  }
}

class SmallCentralWidget extends StatelessWidget {
  final Widget child;
  const SmallCentralWidget({super.key, required this.child});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: 500),
        child: Padding(
          padding: const EdgeInsets.all(20), // Add padding inside the card
          child: child,
        ),
      ),
    );
  }
}

// ─── Common Dialog Styles ──────────────────────────────────────────────────

/// Common dialog style constants
class DialogStyles {
  static const double dialogWidth = 400;
  static const double borderRadius = 20;
  static const EdgeInsets headerPadding = EdgeInsets.fromLTRB(20, 18, 20, 14);
  static const EdgeInsets contentPadding = EdgeInsets.fromLTRB(20, 14, 20, 14);
  static const EdgeInsets footerPadding = EdgeInsets.fromLTRB(20, 12, 20, 16);
  static const double buttonSpacing = 8;
}

/// Small label text for dialog headers and sections (e.g., "SELECT SPEED")
class DialogSectionLabel extends StatelessWidget {
  final String text;
  const DialogSectionLabel(this.text, {super.key});

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    return Text(
      text,
      style: TextStyle(fontSize: 11, letterSpacing: 1.1, color: cs.primary),
    );
  }
}

/// Main title text for dialogs (e.g., "Speed Mode")
class DialogTitle extends StatelessWidget {
  final String text;
  const DialogTitle(this.text, {super.key});

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    return Text(
      text,
      style: TextStyle(
        fontSize: 22,
        fontWeight: FontWeight.w500,
        color: cs.onSurface,
        fontFeatures: const [FontFeature.tabularFigures()],
      ),
    );
  }
}

TextStyle infoTextStyle(BuildContext context) {
  final cs = Theme.of(context).colorScheme;
  ScreenConfiguration screenConfiguration = context.watch();
  double fontSize = screenConfiguration.mode == DisplayMode.vertical ? 9 : 11;
  double letterSpacing =
      screenConfiguration.mode == DisplayMode.vertical ? 0.7 : 1.1;
  return TextStyle(
    fontSize: fontSize,
    letterSpacing: letterSpacing,
    color: cs.onSurface.withValues(alpha: 0.9),
  );
}

TextStyle spinnerTextStyle(BuildContext context) {
  final cs = Theme.of(context).colorScheme;
  ScreenConfiguration screenConfiguration = context.watch();
  double fontSize = screenConfiguration.mode == DisplayMode.vertical ? 12 : 14;
  double letterSpacing =
      screenConfiguration.mode == DisplayMode.vertical ? 0.7 : 1.1;
  return TextStyle(
    fontSize: fontSize,
    fontWeight: FontWeight.w500,
    letterSpacing: letterSpacing,
    color: cs.onSurface,
    fontFeatures: const [FontFeature.tabularFigures()],
  );
}

/// Info text widget for displaying small information (e.g., "from start")
class InfoText extends StatelessWidget {
  final String text;
  const InfoText(this.text, {super.key});

  @override
  Widget build(BuildContext context) {
    return Text(text, textAlign: TextAlign.left, style: infoTextStyle(context));
  }
}

/// Common dialog footer with Cancel and Confirm buttons
class DialogFooter extends StatelessWidget {
  final VoidCallback onCancel;
  final VoidCallback onConfirm;

  const DialogFooter({
    super.key,
    required this.onCancel,
    required this.onConfirm,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: DialogStyles.footerPadding,
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          TextButton(onPressed: onCancel, child: const Text('Cancel')),
          const SizedBox(width: DialogStyles.buttonSpacing),
          FilledButton(onPressed: onConfirm, child: const Text('Confirm')),
        ],
      ),
    );
  }
}

/// Standard dialog container with rounded corners and fixed width
class StandardDialog extends StatelessWidget {
  final List<Widget> sections;

  const StandardDialog({super.key, required this.sections});

  @override
  Widget build(BuildContext context) {
    return Dialog(
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(DialogStyles.borderRadius),
      ),
      child: SizedBox(
        width: DialogStyles.dialogWidth,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: _intersperse(sections, const Divider(height: 1)),
        ),
      ),
    );
  }

  /// Helper to insert dividers between sections
  List<Widget> _intersperse(List<Widget> list, Widget separator) {
    if (list.isEmpty) return list;
    final result = <Widget>[];
    for (int i = 0; i < list.length; i++) {
      result.add(list[i]);
      if (i < list.length - 1) {
        result.add(separator);
      }
    }
    return result;
  }
}

/// Standard dialog header with small label and main title
class DialogHeader extends StatelessWidget {
  final String label;
  final String? title;
  final String? url;
  final List<Widget>? additionalContent;

  const DialogHeader({
    super.key,
    required this.label,
    required this.title,
    required this.url,
    this.additionalContent,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: DialogStyles.headerPadding,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              DialogSectionLabel(label),
              if (url != null) SmallInfoLink(url: url!),
            ],
          ),
          if (title != null) DialogTitle(title!),
          if (additionalContent != null) ...additionalContent!,
        ],
      ),
    );
  }
}
