<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    ZoomIn,
    ZoomOut,
    RotateCcw,
    X,
    Search,
    Network,
    FileText
  } from 'lucide-svelte';
  import Button from './ui/button/Button.svelte';
  import { indexerService, type GraphNode, type GraphLink } from '../services/indexer';

  interface Props {
    activePath?: string;
    onOpenFile: (path: string) => void;
    onClose: () => void;
  }

  let { activePath = '', onOpenFile, onClose }: Props = $props();

  interface SimNode extends GraphNode {
    x: number;
    y: number;
    vx: number;
    vy: number;
    radius: number;
  }

  interface SimEdge {
    sourceId: string;
    targetId: string;
  }

  let canvas = $state<HTMLCanvasElement | null>(null);
  let container = $state<HTMLDivElement | null>(null);
  let searchQuery = $state('');

  // UI Reactive States
  let nodeCount = $state(0);
  let linkCount = $state(0);
  let hoveredNode = $state<SimNode | null>(null);

  // Physics Simulation Data (Plain JS arrays for pure 60fps performance and zero proxy de-sync)
  let simNodes: SimNode[] = [];
  let simEdges: SimEdge[] = [];
  let nodeMap = new Map<string, SimNode>();
  let draggedNode: SimNode | null = null;

  // Camera & Viewport
  let cameraX = 0;
  let cameraY = 0;
  let zoom = 1;
  let isPanning = false;
  let panStartX = 0;
  let panStartY = 0;
  let hasMoved = false;

  let animationFrameId: number;
  let width = 800;
  let height = 600;

  function loadGraph() {
    const data = indexerService.getGraphData();
    nodeMap.clear();

    const count = data.nodes.length;
    const initialRadius = Math.max(140, Math.min(width, height) * 0.32);

    simNodes = data.nodes.map((n, i) => {
      const angle = (i / Math.max(1, count)) * 2 * Math.PI;
      const radius = 6 + Math.min(n.connections * 2, 10);
      const sn: SimNode = {
        ...n,
        x: width / 2 + Math.cos(angle) * initialRadius + (Math.random() - 0.5) * 30,
        y: height / 2 + Math.sin(angle) * initialRadius + (Math.random() - 0.5) * 30,
        vx: 0,
        vy: 0,
        radius
      };
      nodeMap.set(n.id, sn);
      return sn;
    });

    simEdges = data.links.map(l => ({
      sourceId: l.source,
      targetId: l.target
    }));

    nodeCount = simNodes.length;
    linkCount = simEdges.length;
  }

  function tickSimulation() {
    const kRepel = 520;
    const targetDist = 110;
    const kSpring = 0.04;
    const gravity = 0.0006;
    const damping = 0.88;

    const centerX = width / 2;
    const centerY = height / 2;

    // 1. Center Gravity
    for (let i = 0; i < simNodes.length; i++) {
      const n = simNodes[i];
      if (n === draggedNode) continue;
      n.vx += (centerX - n.x) * gravity;
      n.vy += (centerY - n.y) * gravity;
    }

    // 2. Node-Node Repulsion
    for (let i = 0; i < simNodes.length; i++) {
      const n1 = simNodes[i];
      for (let j = i + 1; j < simNodes.length; j++) {
        const n2 = simNodes[j];
        const dx = n2.x - n1.x;
        const dy = n2.y - n1.y;
        const distSq = dx * dx + dy * dy + 100;
        const dist = Math.sqrt(distSq);

        if (dist < 450) {
          const force = kRepel / distSq;
          const fx = (dx / dist) * force;
          const fy = (dy / dist) * force;

          if (n1 !== draggedNode) {
            n1.vx -= fx;
            n1.vy -= fy;
          }
          if (n2 !== draggedNode) {
            n2.vx += fx;
            n2.vy += fy;
          }
        }
      }
    }

    // 3. Link Spring Attraction
    for (let i = 0; i < simEdges.length; i++) {
      const edge = simEdges[i];
      const source = nodeMap.get(edge.sourceId);
      const target = nodeMap.get(edge.targetId);
      if (!source || !target) continue;

      const dx = target.x - source.x;
      const dy = target.y - source.y;
      const dist = Math.sqrt(dx * dx + dy * dy) || 1;
      const force = (dist - targetDist) * kSpring;
      const fx = (dx / dist) * force;
      const fy = (dy / dist) * force;

      if (source !== draggedNode) {
        source.vx += fx;
        source.vy += fy;
      }
      if (target !== draggedNode) {
        target.vx -= fx;
        target.vy -= fy;
      }
    }

    // 4. Update Positions
    for (let i = 0; i < simNodes.length; i++) {
      const n = simNodes[i];
      if (n === draggedNode) continue;
      n.vx *= damping;
      n.vy *= damping;
      n.x += n.vx;
      n.y += n.vy;
    }
  }

  function render() {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    ctx.save();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    // Background
    ctx.fillStyle = '#18181b';
    ctx.fillRect(0, 0, width, height);

    // Camera transform
    ctx.translate(cameraX, cameraY);
    ctx.scale(zoom, zoom);

    // Connected neighbors set for highlight
    const connectedNodeIds = new Set<string>();
    if (hoveredNode) {
      connectedNodeIds.add(hoveredNode.id);
      for (const edge of simEdges) {
        if (edge.sourceId === hoveredNode.id) connectedNodeIds.add(edge.targetId);
        if (edge.targetId === hoveredNode.id) connectedNodeIds.add(edge.sourceId);
      }
    }

    const query = searchQuery.trim().toLowerCase();

    // 1. Draw Links (Directly from live node coordinates)
    for (let i = 0; i < simEdges.length; i++) {
      const edge = simEdges[i];
      const source = nodeMap.get(edge.sourceId);
      const target = nodeMap.get(edge.targetId);
      if (!source || !target) continue;

      const isConnected =
        hoveredNode &&
        (edge.sourceId === hoveredNode.id || edge.targetId === hoveredNode.id);

      ctx.beginPath();
      ctx.moveTo(source.x, source.y);
      ctx.lineTo(target.x, target.y);

      if (isConnected) {
        ctx.strokeStyle = '#38bdf8';
        ctx.lineWidth = 2.2 / zoom;
        ctx.globalAlpha = 0.95;
      } else if (hoveredNode) {
        ctx.strokeStyle = '#27272a';
        ctx.lineWidth = 1 / zoom;
        ctx.globalAlpha = 0.15;
      } else {
        ctx.strokeStyle = '#52525b';
        ctx.lineWidth = 1.4 / zoom;
        ctx.globalAlpha = 0.65;
      }

      ctx.stroke();
    }

    // 2. Draw Nodes
    for (let i = 0; i < simNodes.length; i++) {
      const n = simNodes[i];
      const isHovered = hoveredNode?.id === n.id;
      const isActive = activePath === n.path;
      const isConnected = connectedNodeIds.has(n.id);
      const isMatch = !query || n.name.toLowerCase().includes(query);

      const r = n.radius * (isHovered ? 1.3 : 1);

      // Node Alpha
      if (query && !isMatch) {
        ctx.globalAlpha = 0.2;
      } else if (hoveredNode && !isConnected) {
        ctx.globalAlpha = 0.25;
      } else {
        ctx.globalAlpha = 1;
      }

      // Outer glow for active note or hovered
      if (isActive || isHovered) {
        ctx.beginPath();
        ctx.arc(n.x, n.y, r + 4.5 / zoom, 0, Math.PI * 2);
        ctx.fillStyle = isActive ? 'rgba(56, 189, 248, 0.25)' : 'rgba(56, 189, 248, 0.4)';
        ctx.fill();
      }

      // Node Circle
      ctx.beginPath();
      ctx.arc(n.x, n.y, r, 0, Math.PI * 2);

      if (isActive) {
        ctx.fillStyle = '#38bdf8';
        ctx.strokeStyle = '#e0f2fe';
        ctx.lineWidth = 2 / zoom;
      } else if (isHovered) {
        ctx.fillStyle = '#38bdf8';
        ctx.strokeStyle = '#ffffff';
        ctx.lineWidth = 2 / zoom;
      } else if (isConnected) {
        ctx.fillStyle = '#27272a';
        ctx.strokeStyle = '#38bdf8';
        ctx.lineWidth = 2 / zoom;
      } else {
        ctx.fillStyle = '#27272a';
        ctx.strokeStyle = '#71717a';
        ctx.lineWidth = 1.5 / zoom;
      }

      ctx.fill();
      ctx.stroke();

      // Node Label
      const showLabel = zoom >= 0.6 || isHovered || isActive || isConnected || isMatch;
      if (showLabel) {
        ctx.font = `${Math.max(9, Math.min(12, 11 / zoom))}px 'JetBrains Mono', monospace`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';

        const labelY = n.y + r + 4 / zoom;
        const text = n.name;

        // Label background shadow for contrast
        ctx.fillStyle = 'rgba(18, 18, 21, 0.85)';
        const textWidth = ctx.measureText(text).width;
        ctx.fillRect(
          n.x - textWidth / 2 - 3 / zoom,
          labelY - 1 / zoom,
          textWidth + 6 / zoom,
          13 / zoom
        );

        if (isActive || isHovered) {
          ctx.fillStyle = '#38bdf8';
        } else if (isConnected) {
          ctx.fillStyle = '#f4f4f5';
        } else {
          ctx.fillStyle = '#a1a1aa';
        }

        ctx.fillText(text, n.x, labelY);
      }
    }

    ctx.restore();
  }

  function loop() {
    tickSimulation();
    render();
    animationFrameId = requestAnimationFrame(loop);
  }

  function handleResize() {
    if (!container || !canvas) return;
    const rect = container.getBoundingClientRect();
    width = rect.width;
    height = rect.height;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
  }

  function screenToWorld(sx: number, sy: number) {
    return {
      x: (sx - cameraX) / zoom,
      y: (sy - cameraY) / zoom
    };
  }

  function getNodeAt(x: number, y: number): SimNode | null {
    for (let i = simNodes.length - 1; i >= 0; i--) {
      const n = simNodes[i];
      const dx = n.x - x;
      const dy = n.y - y;
      const hitRadius = (n.radius + 6) / zoom;
      if (dx * dx + dy * dy <= hitRadius * hitRadius) {
        return n;
      }
    }
    return null;
  }

  function onPointerDown(e: PointerEvent) {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const sx = e.clientX - rect.left;
    const sy = e.clientY - rect.top;
    const world = screenToWorld(sx, sy);

    const hit = getNodeAt(world.x, world.y);
    hasMoved = false;

    if (hit) {
      draggedNode = hit;
      draggedNode.vx = 0;
      draggedNode.vy = 0;
    } else {
      isPanning = true;
      panStartX = e.clientX - cameraX;
      panStartY = e.clientY - cameraY;
    }
  }

  function onPointerMove(e: PointerEvent) {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const sx = e.clientX - rect.left;
    const sy = e.clientY - rect.top;
    const world = screenToWorld(sx, sy);

    if (draggedNode) {
      hasMoved = true;
      draggedNode.x = world.x;
      draggedNode.y = world.y;
      draggedNode.vx = 0;
      draggedNode.vy = 0;
      return;
    }

    if (isPanning) {
      hasMoved = true;
      cameraX = e.clientX - panStartX;
      cameraY = e.clientY - panStartY;
      return;
    }

    const hit = getNodeAt(world.x, world.y);
    hoveredNode = hit;
    if (canvas) {
      canvas.style.cursor = hit ? 'pointer' : 'default';
    }
  }

  function onPointerUp(e: PointerEvent) {
    if (draggedNode && !hasMoved) {
      // Clicked on a node without dragging
      onOpenFile(draggedNode.path);
    }
    draggedNode = null;
    isPanning = false;
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const sx = e.clientX - rect.left;
    const sy = e.clientY - rect.top;

    const factor = e.deltaY < 0 ? 1.12 : 0.89;
    const newZoom = Math.max(0.2, Math.min(3.5, zoom * factor));

    // Zoom centered around cursor
    cameraX = sx - ((sx - cameraX) * newZoom) / zoom;
    cameraY = sy - ((sy - cameraY) * newZoom) / zoom;
    zoom = newZoom;
  }

  function resetView() {
    cameraX = 0;
    cameraY = 0;
    zoom = 1;
  }

  function zoomIn() {
    const factor = 1.25;
    const newZoom = Math.min(3.5, zoom * factor);
    cameraX = width / 2 - ((width / 2 - cameraX) * newZoom) / zoom;
    cameraY = height / 2 - ((height / 2 - cameraY) * newZoom) / zoom;
    zoom = newZoom;
  }

  function zoomOut() {
    const factor = 0.8;
    const newZoom = Math.max(0.2, zoom * factor);
    cameraX = width / 2 - ((width / 2 - cameraX) * newZoom) / zoom;
    cameraY = height / 2 - ((height / 2 - cameraY) * newZoom) / zoom;
    zoom = newZoom;
  }

  onMount(async () => {
    handleResize();
    await indexerService.reindexVault();
    loadGraph();
    animationFrameId = requestAnimationFrame(loop);
    window.addEventListener('resize', handleResize);
  });

  onDestroy(() => {
    cancelAnimationFrame(animationFrameId);
    window.removeEventListener('resize', handleResize);
  });
</script>

<div
  bind:this={container}
  class="relative w-full h-full bg-zed-bg-base overflow-hidden select-none"
>
  <canvas
    bind:this={canvas}
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onwheel={onWheel}
    class="w-full h-full block touch-none"
  ></canvas>

  <!-- Top Floating Bar: Search, Stats & Close -->
  <div class="absolute top-3 left-3 right-3 flex items-center justify-between pointer-events-none">
    <div class="flex items-center space-x-2 pointer-events-auto">
      <!-- Search Input -->
      <div class="relative flex items-center">
        <Search size={13} class="absolute left-2.5 text-zinc-500" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter graph nodes..."
          class="h-8 pl-8 pr-3 text-xs bg-zinc-900/90 text-zinc-200 border border-zinc-800 rounded-md shadow-sm focus:outline-none focus:border-zed-accent/80 focus:ring-1 focus:ring-zed-accent/40 w-48 font-mono placeholder:text-zinc-600 transition-all"
        />
        {#if searchQuery}
          <button
            onclick={() => (searchQuery = '')}
            class="absolute right-2 text-zinc-500 hover:text-zinc-300"
            title="Clear filter"
          >
            <X size={12} />
          </button>
        {/if}
      </div>

      <!-- Stats Pill -->
      <div class="h-8 px-3 rounded-md bg-zinc-900/90 border border-zinc-800 flex items-center space-x-1.5 text-[11px] text-zinc-400 font-mono shadow-sm">
        <Network size={12} class="text-zed-accent" />
        <span>{nodeCount} Notes</span>
        <span class="text-zinc-600">•</span>
        <span>{linkCount} Link{linkCount === 1 ? '' : 's'}</span>
      </div>
    </div>

    <!-- Right Controls: Zoom & Close -->
    <div class="flex items-center space-x-1.5 pointer-events-auto bg-zinc-900/90 p-1 rounded-md border border-zinc-800 shadow-sm">
      <Button
        variant="ghost"
        size="icon"
        onclick={zoomIn}
        class="h-6 w-6 text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800"
        title="Zoom In"
      >
        <ZoomIn size={13} />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        onclick={zoomOut}
        class="h-6 w-6 text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800"
        title="Zoom Out"
      >
        <ZoomOut size={13} />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        onclick={resetView}
        class="h-6 w-6 text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800"
        title="Reset Center & Zoom"
      >
        <RotateCcw size={13} />
      </Button>

      <div class="h-4 w-[1px] bg-zinc-800 mx-0.5"></div>

      <Button
        variant="ghost"
        size="icon"
        onclick={onClose}
        class="h-6 w-6 text-zinc-400 hover:text-red-400 hover:bg-zinc-800"
        title="Close Graph View (Esc / Ctrl+G)"
      >
        <X size={14} />
      </Button>
    </div>
  </div>

  <!-- Bottom Floating Tip & Hover Info -->
  <div class="absolute bottom-3 left-3 right-3 flex items-center justify-between pointer-events-none text-[11px]">
    <!-- Bottom Left: Keyboard & Mouse Guide -->
    <div class="px-2.5 py-1 rounded bg-zinc-900/80 border border-zinc-800/80 text-zinc-500 font-mono flex items-center space-x-2 shadow-sm">
      <span>• Click node to open</span>
      <span>• Drag to move</span>
      <span>• Scroll to zoom</span>
    </div>

    <!-- Bottom Right: Active / Hovered Note Details -->
    {#if hoveredNode}
      <div class="px-3 py-1.5 rounded-md bg-zinc-900/95 border border-zed-accent/40 text-zinc-200 font-mono flex items-center space-x-2 shadow-lg animate-in fade-in">
        <FileText size={12} class="text-zed-accent" />
        <span class="font-semibold text-white">{hoveredNode.name}</span>
        <span class="text-zinc-500 text-[10px]">({hoveredNode.connections} connection{hoveredNode.connections === 1 ? '' : 's'})</span>
      </div>
    {/if}
  </div>
</div>
