package com.finova.example

import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.finova.example.ui.theme.FinovaTheme
import com.finova.example.viewmodel.MainViewModel
import com.finova.sdk.*
import com.finova.sdk.models.*
import dagger.hilt.android.AndroidEntryPoint

@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    private val viewModel: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            FinovaTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    MainScreen(viewModel = viewModel)
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(viewModel: MainViewModel) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val context = LocalContext.current
    
    var selectedTab by remember { mutableIntStateOf(0) }
    val tabs = listOf("Mining", "XP", "Referrals", "NFTs", "Profile")
    
    LaunchedEffect(uiState.error) {
        uiState.error?.let { error ->
            Toast.makeText(context, error, Toast.LENGTH_LONG).show()
            viewModel.clearError()
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { 
                    Text("Finova Network", fontWeight = FontWeight.Bold) 
                },
                actions = {
                    IconButton(onClick = { viewModel.refreshData() }) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                    }
                    IconButton(onClick = { viewModel.openSettings() }) {
                        Icon(Icons.Default.Settings, contentDescription = "Settings")
                    }
                }
            )
        },
        bottomBar = {
            NavigationBar {
                tabs.forEachIndexed { index, tab ->
                    NavigationBarItem(
                        icon = { 
                            Icon(
                                when(index) {
                                    0 -> Icons.Default.Diamond
                                    1 -> Icons.Default.Star
                                    2 -> Icons.Default.Group
                                    3 -> Icons.Default.Collections
                                    4 -> Icons.Default.Person
                                    else -> Icons.Default.Home
                                }, 
                                contentDescription = tab
                            ) 
                        },
                        label = { Text(tab) },
                        selected = selectedTab == index,
                        onClick = { selectedTab = index }
                    )
                }
            }
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp)
        ) {
            when (selectedTab) {
                0 -> MiningScreen(uiState, viewModel)
                1 -> XPScreen(uiState, viewModel)
                2 -> ReferralScreen(uiState, viewModel)
                3 -> NFTScreen(uiState, viewModel)
                4 -> ProfileScreen(uiState, viewModel)
            }
            
            if (uiState.isLoading) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator()
                }
            }
        }
    }
}

@Composable
fun MiningScreen(uiState: MainUiState, viewModel: MainViewModel) {
    LazyColumn(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        item {
            MiningStatsCard(uiState.miningData, viewModel)
        }
        item {
            MiningControlsCard(uiState.miningData, viewModel)
        }
        if (uiState.miningHistory.isNotEmpty()) {
            item {
                Text(
                    "Recent Mining History",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold
                )
            }
            items(uiState.miningHistory) { history ->
                MiningHistoryItem(history)
            }
        }
    }
}

@Composable
fun MiningStatsCard(miningData: MiningData?, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    "Mining Status",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold
                )
                miningData?.let {
                    Badge {
                        Text(if (it.isActive) "ACTIVE" else "INACTIVE")
                    }
                }
            }
            
            miningData?.let { data ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Column {
                        Text("Current Rate")
                        Text(
                            "${String.format("%.4f", data.currentRate)} $FIN/hr",
                            style = MaterialTheme.typography.headlineSmall,
                            color = MaterialTheme.colorScheme.primary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                    Column(horizontalAlignment = Alignment.End) {
                        Text("Total Mined")
                        Text(
                            "${String.format("%.2f", data.totalMined)} $FIN",
                            style = MaterialTheme.typography.headlineSmall,
                            color = MaterialTheme.colorScheme.secondary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }
                
                LinearProgressIndicator(
                    progress = data.todayProgress,
                    modifier = Modifier.fillMaxWidth()
                )
                Text(
                    "Daily Progress: ${String.format("%.1f", data.todayProgress * 100)}%",
                    style = MaterialTheme.typography.bodyMedium
                )
            }
        }
    }
}

@Composable
fun MiningControlsCard(miningData: MiningData?, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                "Mining Controls",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold
            )
            
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Button(
                    onClick = { 
                        if (miningData?.isActive == true) {
                            viewModel.stopMining()
                        } else {
                            viewModel.startMining()
                        }
                    },
                    modifier = Modifier.weight(1f)
                ) {
                    Icon(
                        if (miningData?.isActive == true) Icons.Default.Stop else Icons.Default.PlayArrow,
                        contentDescription = null
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(if (miningData?.isActive == true) "Stop Mining" else "Start Mining")
                }
                
                OutlinedButton(
                    onClick = { viewModel.useSpecialCard() },
                    modifier = Modifier.weight(1f)
                ) {
                    Icon(Icons.Default.Star, contentDescription = null)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Use Card")
                }
            }
            
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                OutlinedButton(
                    onClick = { viewModel.claimRewards() },
                    modifier = Modifier.weight(1f)
                ) {
                    Icon(Icons.Default.AccountBalanceWallet, contentDescription = null)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Claim")
                }
                
                OutlinedButton(
                    onClick = { viewModel.openStaking() },
                    modifier = Modifier.weight(1f)
                ) {
                    Icon(Icons.Default.Savings, contentDescription = null)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Stake")
                }
            }
        }
    }
}

@Composable
fun MiningHistoryItem(history: MiningHistory) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column {
                Text(
                    history.timestamp,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Text(
                    "${history.amount} $FIN mined",
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Medium
                )
            }
            Badge {
                Text(history.type.uppercase())
            }
        }
    }
}

@Composable
fun XPScreen(uiState: MainUiState, viewModel: MainViewModel) {
    LazyColumn(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        item {
            XPStatsCard(uiState.xpData, viewModel)
        }
        item {
            XPActivitiesCard(uiState.xpActivities, viewModel)
        }
        item {
            XPLevelProgressCard(uiState.xpData)
        }
    }
}

@Composable
fun XPStatsCard(xpData: XPData?, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                "Experience Points",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold
            )
            
            xpData?.let { data ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Column {
                        Text("Current Level")
                        Text(
                            "${data.currentLevel}",
                            style = MaterialTheme.typography.headlineLarge,
                            color = MaterialTheme.colorScheme.primary,
                            fontWeight = FontWeight.Bold
                        )
                        Text("${data.badgeTier}")
                    }
                    Column(horizontalAlignment = Alignment.End) {
                        Text("Total XP")
                        Text(
                            "${String.format("%,d", data.totalXP)}",
                            style = MaterialTheme.typography.headlineLarge,
                            color = MaterialTheme.colorScheme.secondary,
                            fontWeight = FontWeight.Bold
                        )
                        Text("Mining Bonus: ${data.miningMultiplier}x")
                    }
                }
                
                LinearProgressIndicator(
                    progress = data.levelProgress,
                    modifier = Modifier.fillMaxWidth()
                )
                Text(
                    "Progress to Level ${data.currentLevel + 1}: ${data.xpToNextLevel} XP needed",
                    style = MaterialTheme.typography.bodyMedium
                )
            }
        }
    }
}

@Composable
fun XPActivitiesCard(activities: List<XPActivity>, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    "Daily Activities",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold
                )
                TextButton(onClick = { viewModel.refreshXPActivities() }) {
                    Text("Refresh")
                }
            }
            
            activities.forEach { activity ->
                XPActivityItem(activity, viewModel)
            }
        }
    }
}

@Composable
fun XPActivityItem(activity: XPActivity, viewModel: MainViewModel) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                activity.name,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium
            )
            Text(
                "${activity.baseXP} XP • ${activity.dailyLimit} daily limit",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Text(
                "${activity.completed}/${activity.dailyLimit}",
                style = MaterialTheme.typography.bodyMedium
            )
            Button(
                onClick = { viewModel.completeXPActivity(activity.id) },
                enabled = activity.completed < activity.dailyLimit
            ) {
                Text("Do")
            }
        }
    }
}

@Composable
fun XPLevelProgressCard(xpData: XPData?) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                "Level Benefits",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold
            )
            
            xpData?.let { data ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Mining Boost")
                        Text(
                            "${data.miningMultiplier}x",
                            style = MaterialTheme.typography.headlineSmall,
                            color = MaterialTheme.colorScheme.primary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Daily Cap")
                        Text(
                            "${data.dailyFinCap} $FIN",
                            style = MaterialTheme.typography.headlineSmall,
                            color = MaterialTheme.colorScheme.secondary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }
                
                data.specialUnlocks.forEach { unlock ->
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Icon(
                            Icons.Default.CheckCircle,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.primary
                        )
                        Text(unlock)
                    }
                }
            }
        }
    }
}

@Composable
fun ReferralScreen(uiState: MainUiState, viewModel: MainViewModel) {
    LazyColumn(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        item {
            ReferralStatsCard(uiState.referralData, viewModel)
        }
        item {
            ReferralCodeCard(uiState.referralData, viewModel)
        }
        if (uiState.referrals.isNotEmpty()) {
            item {
                Text(
                    "Your Network",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold
                )
            }
            items(uiState.referrals) { referral ->
                ReferralItem(referral)
            }
        }
    }
}

@Composable
fun ReferralStatsCard(referralData: ReferralData?, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                "Referral Network",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold
            )
            
            referralData?.let { data ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("RP Points")
                        Text(
                            "${String.format("%,d", data.totalRP)}",
                            style = MaterialTheme.typography.headlineLarge,
                            color = MaterialTheme.colorScheme.primary,
                            fontWeight = FontWeight.Bold
                        )
                        Text(data.rpTier)
                    }
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Network Size")
                        Text(
                            "${data.totalReferrals}",
                            style = MaterialTheme.typography.headlineLarge,
                            color = MaterialTheme.colorScheme.secondary,
                            fontWeight = FontWeight.Bold
                        )
                        Text("${data.activeReferrals} active")
                    }
                }
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Network Bonus")
                        Text(
                            "+${data.networkBonus}%",
                            style = MaterialTheme.typography.headlineSmall,
                            color = MaterialTheme.colorScheme.tertiary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Quality Score")
                        Text(
                            "${String.format("%.2f", data.qualityScore)}",
                            style = MaterialTheme.typography.headlineSmall,
                            color = MaterialTheme.colorScheme.tertiary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun ReferralCodeCard(referralData: ReferralData?, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                "Your Referral Code",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold
            )
            
            referralData?.let { data ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        data.referralCode,
                        style = MaterialTheme.typography.headlineSmall,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.primary
                    )
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedButton(
                            onClick = { viewModel.copyReferralCode(data.referralCode) }
                        ) {
                            Icon(Icons.Default.ContentCopy, contentDescription = null)
                            Spacer(modifier = Modifier.width(4.dp))
                            Text("Copy")
                        }
                        Button(
                            onClick = { viewModel.shareReferralCode(data.referralCode) }
                        ) {
                            Icon(Icons.Default.Share, contentDescription = null)
                            Spacer(modifier = Modifier.width(4.dp))
                            Text("Share")
                        }
                    }
                }
                
                Text(
                    "Referral Link: ${data.referralLink}",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
fun ReferralItem(referral: Referral) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column {
                Text(
                    referral.username,
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Medium
                )
                Text(
                    "Level ${referral.level} • Joined ${referral.joinedDate}",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Column(horizontalAlignment = Alignment.End) {
                Badge {
                    Text(if (referral.isActive) "ACTIVE" else "INACTIVE")
                }
                Text(
                    "+${referral.rpEarned} RP",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.primary
                )
            }
        }
    }
}

@Composable
fun NFTScreen(uiState: MainUiState, viewModel: MainViewModel) {
    LazyColumn(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        item {
            NFTStatsCard(uiState.nftData, viewModel)
        }
        if (uiState.nfts.isNotEmpty()) {
            item {
                Text(
                    "Your NFT Collection",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold
                )
            }
            items(uiState.nfts) { nft ->
                NFTItem(nft, viewModel)
            }
        }
    }
}

@Composable
fun NFTStatsCard(nftData: NFTData?, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                "NFT Collection",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold
            )
            
            nftData?.let { data ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Total NFTs")
                        Text(
                            "${data.totalNFTs}",
                            style = MaterialTheme.typography.headlineLarge,
                            color = MaterialTheme.colorScheme.primary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Special Cards")
                        Text(
                            "${data.specialCards}",
                            style = MaterialTheme.typography.headlineLarge,
                            color = MaterialTheme.colorScheme.secondary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Total Value")
                        Text(
                            "${data.totalValue} $FIN",
                            style = MaterialTheme.typography.headlineLarge,
                            color = MaterialTheme.colorScheme.tertiary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Button(
                        onClick = { viewModel.openMarketplace() },
                        modifier = Modifier.weight(1f)
                    ) {
                        Icon(Icons.Default.Store, contentDescription = null)
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Marketplace")
                    }
                    
                    OutlinedButton(
                        onClick = { viewModel.openCardPacks() },
                        modifier = Modifier.weight(1f)
                    ) {
                        Icon(Icons.Default.CardGiftcard, contentDescription = null)
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Buy Packs")
                    }
                }
            }
        }
    }
}

@Composable
fun NFTItem(nft: NFT, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column {
                Text(
                    nft.name,
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Medium
                )
                Text(
                    "${nft.rarity} • ${nft.category}",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                if (nft.effect.isNotEmpty()) {
                    Text(
                        nft.effect,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.primary
                    )
                }
            }
            Column(horizontalAlignment = Alignment.End) {
                if (nft.isUsable) {
                    Button(
                        onClick = { viewModel.useNFT(nft.id) },
                        size = ButtonDefaults.SmallButtonSize
                    ) {
                        Text("Use")
                    }
                }
                Text(
                    "${nft.value} $FIN",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.secondary
                )
            }
        }
    }
}

@Composable
fun ProfileScreen(uiState: MainUiState, viewModel: MainViewModel) {
    LazyColumn(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        item {
            ProfileCard(uiState.userData, viewModel)
        }
        item {
            WalletCard(uiState.walletData, viewModel)
        }
        item {
            AchievementsCard(uiState.achievements, viewModel)
        }
        item {
            SettingsCard(viewModel)
        }
    }
}

@Composable
fun ProfileCard(userData: UserData?, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                "Profile",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold
            )
            
            userData?.let { data ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column {
                        Text(
                            data.username, style = MaterialTheme.typography.headlineSmall,
                            fontWeight = FontWeight.Bold
                        )
                        Text(
                            "@${data.userId}",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Text("Member since ${data.joinedDate}")
                    }
                    Column(horizontalAlignment = Alignment.End) {
                        Badge {
                            Text(data.kycStatus.uppercase())
                        }
                        Text("Human Score: ${String.format("%.2f", data.humanScore)}")
                    }
                }
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Global Rank")
                        Text(
                            "#${data.globalRank}",
                            style = MaterialTheme.typography.headlineSmall,
                            color = MaterialTheme.colorScheme.primary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Total Value")
                        Text(
                            "${String.format("%.2f", data.totalValue)} $FIN",
                            style = MaterialTheme.typography.headlineSmall,
                            color = MaterialTheme.colorScheme.secondary,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }
                
                if (!data.isKYCVerified) {
                    Button(
                        onClick = { viewModel.startKYCVerification() },
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Icon(Icons.Default.Security, contentDescription = null)
                        Spacer(modifier = Modifier.width(8.dp))
                        Text("Complete KYC Verification")
                    }
                }
            }
        }
    }
}

@Composable
fun WalletCard(walletData: WalletData?, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    "Wallet",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold
                )
                IconButton(onClick = { viewModel.refreshWallet() }) {
                    Icon(Icons.Default.Refresh, contentDescription = "Refresh")
                }
            }
            
            walletData?.let { data ->
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    WalletBalanceItem("$FIN", data.finBalance, "Primary Token")
                    WalletBalanceItem("$sFIN", data.sfinBalance, "Staked $FIN")
                    WalletBalanceItem("$USDfin", data.usdfinBalance, "Stablecoin")
                    WalletBalanceItem("SOL", data.solBalance, "Gas Token")
                }
                
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    OutlinedButton(
                        onClick = { viewModel.openDeposit() },
                        modifier = Modifier.weight(1f)
                    ) {
                        Icon(Icons.Default.Add, contentDescription = null)
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("Deposit")
                    }
                    OutlinedButton(
                        onClick = { viewModel.openWithdraw() },
                        modifier = Modifier.weight(1f)
                    ) {
                        Icon(Icons.Default.Remove, contentDescription = null)
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("Withdraw")
                    }
                    OutlinedButton(
                        onClick = { viewModel.openSwap() },
                        modifier = Modifier.weight(1f)
                    ) {
                        Icon(Icons.Default.SwapHoriz, contentDescription = null)
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("Swap")
                    }
                }
            }
        }
    }
}

@Composable
fun WalletBalanceItem(
    token: String,
    balance: Double,
    description: String
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column {
            Text(
                token,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium
            )
            Text(
                description,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        Text(
            String.format("%.6f", balance),
            style = MaterialTheme.typography.bodyLarge,
            fontWeight = FontWeight.Bold
        )
    }
}

@Composable
fun AchievementsCard(achievements: List<Achievement>, viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    "Achievements",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold
                )
                TextButton(onClick = { viewModel.viewAllAchievements() }) {
                    Text("View All")
                }
            }
            
            if (achievements.isNotEmpty()) {
                achievements.take(3).forEach { achievement ->
                    AchievementItem(achievement)
                }
            } else {
                Text(
                    "Complete activities to earn achievements!",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
fun AchievementItem(achievement: Achievement) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Badge(
            containerColor = when (achievement.rarity) {
                "Common" -> MaterialTheme.colorScheme.surfaceVariant
                "Rare" -> MaterialTheme.colorScheme.primary
                "Epic" -> MaterialTheme.colorScheme.secondary
                "Legendary" -> MaterialTheme.colorScheme.tertiary
                else -> MaterialTheme.colorScheme.outline
            }
        ) {
            Text(achievement.icon)
        }
        Column(modifier = Modifier.weight(1f)) {
            Text(
                achievement.name,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium
            )
            Text(
                achievement.description,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        if (achievement.isCompleted) {
            Icon(
                Icons.Default.CheckCircle,
                contentDescription = "Completed",
                tint = MaterialTheme.colorScheme.primary
            )
        }
    }
}

@Composable
fun SettingsCard(viewModel: MainViewModel) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                "Settings",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold
            )
            
            SettingsItem(
                title = "Notifications",
                subtitle = "Push notifications & alerts",
                icon = Icons.Default.Notifications,
                onClick = { viewModel.openNotificationSettings() }
            )
            
            SettingsItem(
                title = "Security",
                subtitle = "PIN, biometrics & privacy",
                icon = Icons.Default.Security,
                onClick = { viewModel.openSecuritySettings() }
            )
            
            SettingsItem(
                title = "Social Accounts",
                subtitle = "Connect platforms",
                icon = Icons.Default.Link,
                onClick = { viewModel.openSocialSettings() }
            )
            
            SettingsItem(
                title = "Help & Support",
                subtitle = "FAQ, contact us",
                icon = Icons.Default.Help,
                onClick = { viewModel.openSupport() }
            )
            
            SettingsItem(
                title = "About",
                subtitle = "Version, terms & privacy",
                icon = Icons.Default.Info,
                onClick = { viewModel.openAbout() }
            )
        }
    }
}

@Composable
fun SettingsItem(
    title: String,
    subtitle: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
            .padding(vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            icon,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary
        )
        Column(modifier = Modifier.weight(1f)) {
            Text(
                title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium
            )
            Text(
                subtitle,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        Icon(
            Icons.Default.ChevronRight,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

// Data classes for UI state
data class MainUiState(
    val isLoading: Boolean = false,
    val error: String? = null,
    val miningData: MiningData? = null,
    val miningHistory: List<MiningHistory> = emptyList(),
    val xpData: XPData? = null,
    val xpActivities: List<XPActivity> = emptyList(),
    val referralData: ReferralData? = null,
    val referrals: List<Referral> = emptyList(),
    val nftData: NFTData? = null,
    val nfts: List<NFT> = emptyList(),
    val userData: UserData? = null,
    val walletData: WalletData? = null,
    val achievements: List<Achievement> = emptyList()
)

data class MiningData(
    val isActive: Boolean,
    val currentRate: Double,
    val totalMined: Double,
    val todayProgress: Float,
    val phase: String,
    val multipliers: Map<String, Double>
)

data class MiningHistory(
    val timestamp: String,
    val amount: Double,
    val type: String
)

data class XPData(
    val currentLevel: Int,
    val totalXP: Int,
    val badgeTier: String,
    val levelProgress: Float,
    val xpToNextLevel: Int,
    val miningMultiplier: Double,
    val dailyFinCap: Double,
    val specialUnlocks: List<String>
)

data class XPActivity(
    val id: String,
    val name: String,
    val baseXP: Int,
    val dailyLimit: Int,
    val completed: Int,
    val platform: String
)

data class ReferralData(
    val totalRP: Int,
    val rpTier: String,
    val totalReferrals: Int,
    val activeReferrals: Int,
    val networkBonus: Double,
    val qualityScore: Double,
    val referralCode: String,
    val referralLink: String
)

data class Referral(
    val username: String,
    val level: Int,
    val joinedDate: String,
    val isActive: Boolean,
    val rpEarned: Int
)

data class NFTData(
    val totalNFTs: Int,
    val specialCards: Int,
    val totalValue: Double
)

data class NFT(
    val id: String,
    val name: String,
    val rarity: String,
    val category: String,
    val effect: String,
    val value: Double,
    val isUsable: Boolean
)

data class UserData(
    val username: String,
    val userId: String,
    val joinedDate: String,
    val kycStatus: String,
    val isKYCVerified: Boolean,
    val humanScore: Double,
    val globalRank: Int,
    val totalValue: Double
)

data class WalletData(
    val finBalance: Double,
    val sfinBalance: Double,
    val usdfinBalance: Double,
    val solBalance: Double
)

data class Achievement(
    val name: String,
    val description: String,
    val icon: String,
    val rarity: String,
    val isCompleted: Boolean,
    val reward: String
)
