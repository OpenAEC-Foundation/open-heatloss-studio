import { describe, expect, it } from "vitest";

import {
  DEFAULT_UNHEATED_FACTOR,
  collectUnheatedTargetIds,
  isUnheatedTarget,
  resolveUnheatedRoomIds,
} from "./isso53Unheated";
import type { ConstructionElement, Room } from "../types/project";
import {
  DEFAULT_ISSO53_ROOM,
  type Isso53RoomState,
} from "../types/projectV2";

function makeElement(
  overrides: Partial<ConstructionElement> = {},
): ConstructionElement {
  return {
    id: "ce-1",
    description: "Wand",
    area: 10,
    u_value: 0.3,
    boundary_type: "exterior",
    material_type: "masonry",
    ...overrides,
  };
}

function makeRoom(overrides: Partial<Room> = {}): Room {
  return {
    id: "room-1",
    name: "Test",
    function: "living_room",
    floor_area: 20,
    constructions: [],
    heating_system: "radiator_ht",
    ...overrides,
  };
}

describe("collectUnheatedTargetIds", () => {
  it("verzamelt adjacent_room_id van unheated_space-constructies", () => {
    const rooms: Room[] = [
      makeRoom({
        id: "K01",
        constructions: [
          makeElement({ boundary_type: "exterior" }),
          makeElement({
            id: "to-meterkast",
            boundary_type: "unheated_space",
            adjacent_room_id: "MK",
          }),
        ],
      }),
      makeRoom({
        id: "K02",
        constructions: [
          makeElement({
            id: "to-berging",
            boundary_type: "unheated_space",
            adjacent_room_id: "BRG",
          }),
        ],
      }),
    ];
    expect(collectUnheatedTargetIds(rooms)).toEqual(new Set(["MK", "BRG"]));
  });

  it("dedupliceert wanneer meerdere ruimtes naar hetzelfde doel grenzen", () => {
    const rooms: Room[] = [
      makeRoom({
        id: "K01",
        constructions: [
          makeElement({
            boundary_type: "unheated_space",
            adjacent_room_id: "MK",
          }),
        ],
      }),
      makeRoom({
        id: "K02",
        constructions: [
          makeElement({
            boundary_type: "unheated_space",
            adjacent_room_id: "MK",
          }),
        ],
      }),
    ];
    expect(collectUnheatedTargetIds(rooms)).toEqual(new Set(["MK"]));
  });

  it("negeert unheated_space zonder adjacent_room_id en andere boundary-types", () => {
    const rooms: Room[] = [
      makeRoom({
        constructions: [
          makeElement({ boundary_type: "unheated_space" }), // geen adjacent
          makeElement({
            boundary_type: "adjacent_room",
            adjacent_room_id: "K99",
          }),
        ],
      }),
    ];
    expect(collectUnheatedTargetIds(rooms)).toEqual(new Set());
  });
});

describe("isUnheatedTarget", () => {
  const rooms: Room[] = [
    makeRoom({
      id: "K01",
      constructions: [
        makeElement({
          boundary_type: "unheated_space",
          adjacent_room_id: "MK",
        }),
      ],
    }),
  ];

  it("true voor een onverwarmd-doel-ruimte", () => {
    expect(isUnheatedTarget(rooms, "MK")).toBe(true);
  });

  it("false voor een gewone ruimte", () => {
    expect(isUnheatedTarget(rooms, "K01")).toBe(false);
  });
});

describe("DEFAULT_UNHEATED_FACTOR", () => {
  it("is 0,5 (isso51-consistent)", () => {
    expect(DEFAULT_UNHEATED_FACTOR).toBe(0.5);
  });
});

describe("resolveUnheatedRoomIds", () => {
  const rooms: Room[] = [
    makeRoom({
      id: "K01",
      constructions: [
        makeElement({
          boundary_type: "unheated_space",
          adjacent_room_id: "MK", // impliciet onverwarmd-doel
        }),
      ],
    }),
    makeRoom({ id: "TECH" }),
    makeRoom({ id: "AFVAL" }),
  ];

  it("vereent impliciete unheated_space-doelen met expliciete isUnheated-flags", () => {
    const isso53Rooms: Record<string, Isso53RoomState> = {
      TECH: { ...DEFAULT_ISSO53_ROOM, isUnheated: true },
      AFVAL: { ...DEFAULT_ISSO53_ROOM, isUnheated: true },
    };
    expect(resolveUnheatedRoomIds(rooms, isso53Rooms)).toEqual(
      new Set(["MK", "TECH", "AFVAL"]),
    );
  });

  it("negeert sidecars zonder isUnheated en levert puur de impliciete doelen", () => {
    const isso53Rooms: Record<string, Isso53RoomState> = {
      TECH: { ...DEFAULT_ISSO53_ROOM, isUnheated: false },
    };
    expect(resolveUnheatedRoomIds(rooms, isso53Rooms)).toEqual(new Set(["MK"]));
  });

  it("lege sidecar-map → alleen impliciete doelen", () => {
    expect(resolveUnheatedRoomIds(rooms, {})).toEqual(new Set(["MK"]));
  });
});
