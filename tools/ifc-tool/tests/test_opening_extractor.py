"""Tests for opening extractor — wall matching and geometry logic."""

from __future__ import annotations

import pytest

from ifc_tool.import_ifc.opening_extractor import (
    _match_to_room_wall,
    _point_to_segment,
)
from ifc_tool.models import ModelDoor, ModelRoom, ModelWindow, Point2D


# ---------------------------------------------------------------------------
# _point_to_segment tests
# ---------------------------------------------------------------------------


class TestPointToSegment:
    def test_point_on_segment(self) -> None:
        """Point exactly on the segment → distance 0."""
        a = Point2D(x=0, y=0)
        b = Point2D(x=4000, y=0)
        p = Point2D(x=2000, y=0)
        dist, offset = _point_to_segment(p, a, b)
        assert dist == pytest.approx(0.0)
        assert offset == pytest.approx(2000.0)

    def test_point_above_midpoint(self) -> None:
        """Point 100mm above the midpoint of a horizontal segment."""
        a = Point2D(x=0, y=0)
        b = Point2D(x=4000, y=0)
        p = Point2D(x=2000, y=100)
        dist, offset = _point_to_segment(p, a, b)
        assert dist == pytest.approx(100.0)
        assert offset == pytest.approx(2000.0)

    def test_point_at_start(self) -> None:
        """Point projected onto the start of the segment."""
        a = Point2D(x=0, y=0)
        b = Point2D(x=4000, y=0)
        p = Point2D(x=0, y=50)
        dist, offset = _point_to_segment(p, a, b)
        assert dist == pytest.approx(50.0)
        assert offset == pytest.approx(0.0)

    def test_point_past_end(self) -> None:
        """Point beyond the end of the segment → clamped to end."""
        a = Point2D(x=0, y=0)
        b = Point2D(x=4000, y=0)
        p = Point2D(x=5000, y=0)
        dist, offset = _point_to_segment(p, a, b)
        assert dist == pytest.approx(1000.0)
        assert offset == pytest.approx(4000.0)

    def test_vertical_segment(self) -> None:
        """Vertical segment with point offset horizontally."""
        a = Point2D(x=0, y=0)
        b = Point2D(x=0, y=3000)
        p = Point2D(x=25, y=1500)
        dist, offset = _point_to_segment(p, a, b)
        assert dist == pytest.approx(25.0)
        assert offset == pytest.approx(1500.0)

    def test_zero_length_segment(self) -> None:
        """Degenerate segment (zero length)."""
        a = Point2D(x=1000, y=1000)
        b = Point2D(x=1000, y=1000)
        p = Point2D(x=1100, y=1000)
        dist, offset = _point_to_segment(p, a, b)
        assert dist == pytest.approx(100.0)
        assert offset == pytest.approx(0.0)


# ---------------------------------------------------------------------------
# _match_to_room_wall tests
# ---------------------------------------------------------------------------


class TestMatchToRoomWall:
    def test_match_to_south_wall(self, sample_room: ModelRoom) -> None:
        """Point near the bottom edge (y=0) → wall 0."""
        # Room polygon: (0,0)→(4000,0)→(4000,3000)→(0,3000)
        pos = Point2D(x=2000, y=10)  # 10mm above wall 0
        result = _match_to_room_wall(pos, [sample_room])
        assert result is not None
        room_idx, wall_index, offset = result
        assert room_idx == 0
        assert wall_index == 0
        assert offset == pytest.approx(2000.0)

    def test_match_to_east_wall(self, sample_room: ModelRoom) -> None:
        """Point near the right edge (x=4000) → wall 1."""
        pos = Point2D(x=3990, y=1500)
        result = _match_to_room_wall(pos, [sample_room])
        assert result is not None
        room_idx, wall_index, offset = result
        assert room_idx == 0
        assert wall_index == 1
        assert offset == pytest.approx(1500.0)

    def test_match_to_north_wall(self, sample_room: ModelRoom) -> None:
        """Point near the top edge (y=3000) → wall 2."""
        pos = Point2D(x=2000, y=2990)
        result = _match_to_room_wall(pos, [sample_room])
        assert result is not None
        room_idx, wall_index, offset = result
        assert room_idx == 0
        assert wall_index == 2

    def test_match_to_west_wall(self, sample_room: ModelRoom) -> None:
        """Point near the left edge (x=0) → wall 3."""
        pos = Point2D(x=10, y=1500)
        result = _match_to_room_wall(pos, [sample_room])
        assert result is not None
        room_idx, wall_index, offset = result
        assert room_idx == 0
        assert wall_index == 3

    def test_no_match_too_far(self, sample_room: ModelRoom) -> None:
        """Point too far from any wall → None."""
        pos = Point2D(x=2000, y=1500)  # Center of room, >50mm from walls
        result = _match_to_room_wall(pos, [sample_room])
        assert result is None

    def test_multiple_rooms(self) -> None:
        """Match to the correct room when multiple rooms exist."""
        room_a = ModelRoom(
            name="Room A",
            function="custom",
            polygon=[
                Point2D(x=0, y=0),
                Point2D(x=3000, y=0),
                Point2D(x=3000, y=3000),
                Point2D(x=0, y=3000),
            ],
            floor=0,
            height=2600,
        )
        room_b = ModelRoom(
            name="Room B",
            function="custom",
            polygon=[
                Point2D(x=5000, y=0),
                Point2D(x=8000, y=0),
                Point2D(x=8000, y=3000),
                Point2D(x=5000, y=3000),
            ],
            floor=0,
            height=2600,
        )
        # Point near Room B's south wall
        pos = Point2D(x=6500, y=10)
        result = _match_to_room_wall(pos, [room_a, room_b])
        assert result is not None
        room_idx, wall_index, offset = result
        assert room_idx == 1  # Room B
        assert wall_index == 0


# ---------------------------------------------------------------------------
# ModelWindow / ModelDoor with height and sillHeight
# ---------------------------------------------------------------------------


class TestOpeningModels:
    def test_window_with_height_and_sill(self) -> None:
        """Window with height and sill height serializes correctly."""
        window = ModelWindow(
            room_id="0.01",
            wall_index=0,
            offset=1500,
            width=1200,
            height=1400,
            sill_height=900,
        )
        data = window.model_dump(by_alias=True)
        assert data["roomId"] == "0.01"
        assert data["width"] == 1200
        assert data["height"] == 1400
        assert data["sillHeight"] == 900

    def test_window_without_height(self) -> None:
        """Window without height → None in output."""
        window = ModelWindow(
            room_id="0.01",
            wall_index=0,
            offset=1500,
            width=900,
        )
        data = window.model_dump(by_alias=True)
        assert data["height"] is None
        assert data["sillHeight"] is None

    def test_door_with_height(self) -> None:
        """Door with height serializes correctly."""
        door = ModelDoor(
            room_id="0.01",
            wall_index=1,
            offset=500,
            width=900,
            height=2100,
            swing="left",
        )
        data = door.model_dump(by_alias=True)
        assert data["width"] == 900
        assert data["height"] == 2100
        assert data["swing"] == "left"

    def test_door_without_height(self) -> None:
        """Door without height → None in output."""
        door = ModelDoor(
            room_id="0.01",
            wall_index=0,
            offset=1000,
            width=800,
            swing="right",
        )
        data = door.model_dump(by_alias=True)
        assert data["height"] is None
