import math

# Given P range: from 2^(-64) to 2^64
P_min = 2 ** (-64)
P_max = 2 ** 64

# Calculate the range for i
i_min = math.log(P_min) / math.log(1.0001)
i_max = math.log(P_max) / math.log(1.0001)

print(i_min, i_max)
