#!/usr/bin/env python2

import sys
import time
import curses


def main(window):
    curses.init_pair(1, curses.COLOR_BLACK, curses.COLOR_BLUE)
    curses.init_pair(2, curses.COLOR_BLACK, curses.COLOR_RED)

    height, width = window.getmaxyx()

    window.addstr(0, 0, 'height: %s' % height)
    window.refresh()

    #width /= 2

    while True:
        line = sys.stdin.readline().strip()
        if 'Running' in line:
            continue
        if not line:
            return
        fft_avgs = [float(x.strip()) for x in line.split(',')]
        if len(fft_avgs) > width:
            step = len(fft_avgs)/width + 1
            fft_avgs = [sum(fft_avgs[i:i+step])/step for i in xrange(0, len(fft_avgs)-step, step)]
            if len(fft_avgs) >= width:
                raise Exception('that didnt work')

        highest = max(fft_avgs)
        lowest = min(fft_avgs)

        #highest = 30.0
        if highest == 0:
            highest = 1
        divider = highest - lowest
        #divider = highest
        if divider == 0:
            divider == 2

        heights = [int((x-lowest)/divider*height) for x in fft_avgs]

        for row in range(height):
            eq_line = ''
            for i, f in enumerate(heights):
                if f > (height - row):
                    eq_line = ' '
                    color = curses.A_STANDOUT
                else:
                    eq_line = ' '
                    color = 1
                window.addstr(row, i, eq_line, color)
        #window.addstr(1,0, 'a: %s' % (fft_avgs,))
        window.refresh()


if __name__ == '__main__':
    curses.wrapper(main)
