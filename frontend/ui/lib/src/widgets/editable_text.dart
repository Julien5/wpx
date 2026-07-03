import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

class WritableText extends StatefulWidget {
  final String initialName;
  final ValueChanged<String> onSubmitted;

  const WritableText({
    super.key,
    required this.initialName,
    required this.onSubmitted,
  });

  @override
  State<WritableText> createState() => _WritableTextState();
}

class _WritableTextState extends State<WritableText> {
  late final TextEditingController _controller = TextEditingController(
    text: widget.initialName,
  );
  late final FocusNode _focusNode =
      FocusNode()..addListener(() {
        if (!_focusNode.hasFocus && _isEditing) _commit();
      });
  bool _isEditing = false;
  bool _isHovering = false;

  @override
  void dispose() {
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void _startEditing() {
    setState(() => _isEditing = true);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _focusNode.requestFocus();
      _controller.selection = TextSelection(
        baseOffset: 0,
        extentOffset: _controller.text.length,
      );
    });
  }

  void _commit() {
    final newName = _controller.text.trim();
    setState(() => _isEditing = false);
    if (newName.isNotEmpty && newName != widget.initialName) {
      widget.onSubmitted(newName);
    } else {
      _controller.text = widget.initialName;
    }
  }

  void _cancel() {
    _controller.text = widget.initialName;
    setState(() => _isEditing = false);
    _focusNode.unfocus();
  }

  @override
  Widget build(BuildContext context) {
    if (_isEditing) {
      return SizedBox(
        width: 200,
        child: KeyboardListener(
          focusNode: FocusNode(),
          onKeyEvent: (event) {
            if (event is KeyDownEvent &&
                event.logicalKey == LogicalKeyboardKey.escape) {
              _cancel();
            }
          },
          child: TextField(
            controller: _controller,
            focusNode: _focusNode,
            style: Theme.of(context).textTheme.bodyMedium,
            decoration: const InputDecoration(
              isDense: true,
              contentPadding: EdgeInsets.symmetric(vertical: 4),
              border: UnderlineInputBorder(),
            ),
            onSubmitted: (_) => _commit(),
          ),
        ),
      );
    }

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: _startEditing,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(widget.initialName),
            const SizedBox(width: 6),
            Opacity(
              opacity: _isHovering ? 1 : 0.4,
              child: const Icon(Icons.edit, size: 14),
            ),
          ],
        ),
      ),
    );
  }
}
