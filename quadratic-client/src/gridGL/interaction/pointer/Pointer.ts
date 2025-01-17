import { Viewport } from 'pixi-viewport';
import { InteractionEvent } from 'pixi.js';
import { pixiApp } from '../../pixiApp/PixiApp';
import { PointerAutoComplete } from './PointerAutoComplete/PointerAutoComplete';
import { PointerDown } from './PointerDown';
import { PointerHeading } from './PointerHeading';
import { PointerCursor } from './pointerCursor';

export class Pointer {
  pointerHeading: PointerHeading;
  pointerAutoComplete: PointerAutoComplete;
  private pointerCursor: PointerCursor;

  pointerDown: PointerDown;

  constructor(viewport: Viewport) {
    this.pointerHeading = new PointerHeading();
    this.pointerAutoComplete = new PointerAutoComplete();
    this.pointerDown = new PointerDown();
    this.pointerCursor = new PointerCursor();

    viewport.on('pointerdown', this.handlePointerDown);
    viewport.on('pointermove', this.pointerMove);
    viewport.on('pointerup', this.pointerUp);
    viewport.on('pointerupoutside', this.pointerUp);
  }

  destroy() {
    const viewport = pixiApp.viewport;
    viewport.off('pointerdown', this.handlePointerDown);
    viewport.off('pointermove', this.pointerMove);
    viewport.off('pointerup', this.pointerUp);
    viewport.off('pointerupoutside', this.pointerUp);
    this.pointerDown.destroy();
  }

  // check if more than one touch point (let the viewport handle the event)
  private isMoreThanOneTouch(e: InteractionEvent): boolean {
    return e.data.pointerType === 'touch' && (e.data.originalEvent as TouchEvent).touches.length > 1;
  }

  private handlePointerDown = (e: InteractionEvent): void => {
    if (this.isMoreThanOneTouch(e)) return;
    const world = pixiApp.viewport.toWorld(e.data.global);
    const event = e.data.originalEvent as PointerEvent;
    this.pointerHeading.pointerDown(world, event) ||
      this.pointerAutoComplete.pointerDown(world) ||
      this.pointerDown.pointerDown(world, event);
  };

  private pointerMove = (e: InteractionEvent): void => {
    if (this.isMoreThanOneTouch(e)) return;
    const world = pixiApp.viewport.toWorld(e.data.global);
    this.pointerHeading.pointerMove(world) ||
      this.pointerAutoComplete.pointerMove(world) ||
      this.pointerDown.pointerMove(world);
    this.pointerCursor.pointerMove();
  };

  private pointerUp = (e: InteractionEvent): void => {
    if (this.isMoreThanOneTouch(e)) return;
    this.pointerHeading.pointerUp() || this.pointerAutoComplete.pointerUp() || this.pointerDown.pointerUp();
  };

  handleEscape(): boolean {
    return this.pointerHeading.handleEscape() || this.pointerAutoComplete.handleEscape();
  }
}
