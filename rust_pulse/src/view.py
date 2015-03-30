#!/usr/bin/env python2

import sys
import time
import curses


def main(window):
    height, width = window.getmaxyx()

    window.addstr(0, 0, 'height: %s' % height)
    window.refresh()

    #width /= 2

    while True:
        line = sys.stdin.readline().strip()
        if not line:
            return
        fft_avgs = [float(x.strip()) for x in line.split(',')]
        if len(fft_avgs) > width:
            step = len(fft_avgs)/width + 1
            fft_avgs = [sum(fft_avgs[i:i+step])/step for i in xrange(0, len(fft_avgs)-step, step)]
            if len(fft_avgs) >= width:
                raise Exception('that didnt work')

        highest = max(fft_avgs)
        if highest == 0:
            highest = 1
        heights = [int(x/highest*height) for x in fft_avgs]
        for row in range(height):
            eq_line = ''
            for i, f in enumerate(heights):
                if f > (height - row):
                    eq_line = '$'
                else:
                    eq_line = ' '
                window.addstr(row, i, eq_line)
        window.refresh()


if __name__ == '__main__':
    curses.wrapper(main)
