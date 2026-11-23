using System;
using System.Collections.Generic;
using System.Linq;

public static class NodeEventSchedule
{
    public const int DefaultMaxEvents = 100_000;
    private const ulong Gamma = 0x9e3779b97f4a7c15UL;
    private const long ExactIndexLimit = 1L << 53;

    public readonly struct Event
    {
        public readonly int Node;
        public readonly double Time;
        public readonly ulong Key;

        public Event(int node, double time, ulong key)
        {
            Node = node;
            Time = time;
            Key = key;
        }
    }

    public static TPoint[] CacheNodes<TNode, TPoint>(IEnumerable<TNode> nodes, Func<TNode, TPoint> transform)
        => nodes.Select(transform).ToArray();

    /// <summary>Fills one absolute window. Nodes are sampled uniformly with replacement.
    /// A rejected request leaves the output empty.</summary>
    public static void PlanWindow(ulong seed, long window, double duration, double eventsPerSecond,
        int nodeCount, List<Event> output, int maxEvents = DefaultMaxEvents)
    {
        if (output == null)
            throw new ArgumentNullException(nameof(output));
        output.Clear();
        if (!(duration >= 0) || double.IsInfinity(duration))
            throw new ArgumentOutOfRangeException(nameof(duration));
        if (!(eventsPerSecond >= 0) || double.IsInfinity(eventsPerSecond))
            throw new ArgumentOutOfRangeException(nameof(eventsPerSecond));
        if (nodeCount < 0)
            throw new ArgumentOutOfRangeException(nameof(nodeCount));
        if (maxEvents < 0)
            throw new ArgumentOutOfRangeException(nameof(maxEvents));
        if (duration == 0)
            return;
        if (window < -ExactIndexLimit || window >= ExactIndexLimit)
            throw new ArgumentOutOfRangeException(nameof(window));
        double start = window * duration;
        double end = (window + 1) * duration;
        if (double.IsInfinity(start) || double.IsInfinity(end) || end <= start)
            throw new ArgumentOutOfRangeException(nameof(window), "Window boundaries must be finite and distinct.");
        if (eventsPerSecond == 0 || nodeCount == 0)
            return;

        ulong word = Mix(seed ^ Mix(unchecked((ulong)window)));
        ulong bound = (ulong)nodeCount;
        ulong threshold = unchecked(0UL - bound) % bound;
        double offset = 0;
        while (true)
        {
            // Exponential interarrival times give a Poisson count, already in time order.
            word = unchecked(word + Gamma);
            offset -= Math.Log(Open01(Mix(word))) / eventsPerSecond;
            if (offset >= duration)
                return;
            if (output.Count == maxEvents)
            {
                output.Clear();
                throw new InvalidOperationException("The window exceeds the requested event limit.");
            }
            ulong key = Mix(word ^ 0xd1b54a32d192ed03UL);
            ulong bits = Mix(key ^ 0x94d049bb133111ebUL);
            while (bits < threshold)
                bits = Mix(unchecked(bits + Gamma));
            double time = start + offset;
            if (time >= end)
                time = PreviousDouble(end);
            output.Add(new Event((int)(bits % bound), time, key));
        }
    }

    /// <summary>Samples a payload channel in (0, 1) independently of polling and earlier events.</summary>
    public static double Unit(ulong eventKey, ulong channel)
        => Open01(Mix(unchecked(eventKey + Gamma * (channel + 1))));

    private static double Open01(ulong bits) => ((bits >> 12) + 0.5) / 4503599627370496.0;

    private static double PreviousDouble(double value)
    {
        if (value == 0)
            return -double.Epsilon;
        long bits = BitConverter.DoubleToInt64Bits(value);
        return BitConverter.Int64BitsToDouble(bits + (value > 0 ? -1 : 1));
    }

    private static ulong Mix(ulong value)
    {
        unchecked
        {
            value = (value ^ (value >> 30)) * 0xbf58476d1ce4e5b9UL;
            value = (value ^ (value >> 27)) * 0x94d049bb133111ebUL;
            return value ^ (value >> 31);
        }
    }
}
